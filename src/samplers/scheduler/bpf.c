// Based on: https://github.com/iovisor/bcc/blob/master/tools/runqlat.py

#include <uapi/linux/ptrace.h>
#include <linux/sched.h>
#include <linux/nsproxy.h>
#include <linux/pid_namespace.h>

typedef struct pid_key {
    u64 id;
    u64 slot;
} pid_key_t;

typedef struct pidns_key {
    u64 id;
    u64 slot;
} pidns_key_t;

BPF_TABLE("hash", u32, u64, runqueue_start, 65536);
BPF_HASH(throttle_start, int);

// value_to_index() gives us from 0-460 as the index
BPF_HISTOGRAM(runqueue_latency, int, 461);
BPF_HISTOGRAM(cfs_throttle, int, 461);

struct rq;

// from /sys/kernel/debug/tracing/events/sched/sched_wakeup/format
struct sched_wakeup_arg {
    u64 __unused__;
    char comm[16];
    pid_t pid;
    int prio;
    int success;
    int target_cpu;
};

typedef struct cgroup_key {
    char name[64];
} cgroup_key_t;

/* type-specific structures for kernfs_node union members */
struct kernfs_elem_dir {
    unsigned long       subdirs;
    /* children rbtree starts here and goes through kn->rb */
    struct rb_root      children;

    /*
     * The kernfs hierarchy this directory belongs to.  This fits
     * better directly in kernfs_node but is here to save space.
     */
    struct kernfs_root  *root;
};

struct kernfs_elem_symlink {
    struct kernfs_node  *target_kn;
};

struct kernfs_elem_attr {
    const struct kernfs_ops *ops;
    struct kernfs_open_node *open;
    loff_t          size;
    struct kernfs_node  *notify_next;   /* for kernfs_notify() */
};


struct kernfs_node {
    atomic_t        count;
    atomic_t        active;

    /*
     * Use kernfs_get_parent() and kernfs_name/path() instead of
     * accessing the following two fields directly.  If the node is
     * never moved to a different parent, it is safe to access the
     * parent directly.
     */
    struct kernfs_node  *parent;
    const char      *name;

    struct rb_node      rb;

    const void      *ns;    /* namespace tag */
    unsigned int        hash;   /* ns + name hash */
    union {
        struct kernfs_elem_dir      dir;
        struct kernfs_elem_symlink  symlink;
        struct kernfs_elem_attr     attr;
    };

    void            *priv;

    /*
     * 64bit unique ID.  On 64bit ino setups, id is the ino.  On 32bit,
     * the low 32bits are ino and upper generation.
     */
    u64         id;

    unsigned short      flags;
    umode_t         mode;
    struct kernfs_iattrs    *iattr;
};

struct rt_bandwidth {
    /* nests inside the rq lock: */
    raw_spinlock_t      rt_runtime_lock;
    ktime_t         rt_period;
    u64         rt_runtime;
    struct hrtimer      rt_period_timer;
    unsigned int        rt_period_active;
};

struct cfs_bandwidth {
    raw_spinlock_t      lock;
    ktime_t         period;
    u64         quota;
    u64         runtime;
    s64         hierarchical_quota;

    u8          idle;
    u8          period_active;
    u8          distribute_running;
    u8          slack_started;
    struct hrtimer      period_timer;
    struct hrtimer      slack_timer;
    struct list_head    throttled_cfs_rq;

    /* Statistics: */
    int         nr_periods;
    int         nr_throttled;
    u64         throttled_time;
};

struct rcu_work {
    struct work_struct work;
    struct rcu_head rcu;

    /* target workqueue ->rcu uses to queue ->work */
    struct workqueue_struct *wq;
};

/*
 * Per-subsystem/per-cgroup state maintained by the system.  This is the
 * fundamental structural building block that controllers deal with.
 *
 * Fields marked with "PI:" are public and immutable and may be accessed
 * directly without synchronization.
 */
struct cgroup_subsys_state {
    /* PI: the cgroup that this css is attached to */
    struct cgroup *cgroup;

    /* PI: the cgroup subsystem that this css is attached to */
    struct cgroup_subsys *ss;

    /* reference count - access via css_[try]get() and css_put() */
    struct percpu_ref refcnt;

    /* siblings list anchored at the parent's ->children */
    struct list_head sibling;
    struct list_head children;

    /* flush target list anchored at cgrp->rstat_css_list */
    struct list_head rstat_css_node;

    /*
     * PI: Subsys-unique ID.  0 is unused and root is always 1.  The
     * matching css can be looked up using css_from_id().
     */
    int id;

    unsigned int flags;

    /*
     * Monotonically increasing unique serial number which defines a
     * uniform order among all csses.  It's guaranteed that all
     * ->children lists are in the ascending order of ->serial_nr and
     * used to allow interrupting and resuming iterations.
     */
    u64 serial_nr;

    /*
     * Incremented by online self and children.  Used to guarantee that
     * parents are not offlined before their children.
     */
    atomic_t online_cnt;

    /* percpu_ref killing and RCU release */
    struct work_struct destroy_work;
    struct rcu_work destroy_rwork;

    /*
     * PI: the parent css.  Placed here for cache proximity to following
     * fields of the containing structure.
     */
    struct cgroup_subsys_state *parent;
};


/* Task group related information */
struct task_group {
    struct cgroup_subsys_state css;

    /* schedulable entities of this group on each CPU */
    struct sched_entity **se;
    /* runqueue "owned" by this group on each CPU */
    struct cfs_rq       **cfs_rq;
    unsigned long       shares;

    /*
     * load_avg can be heavily contended at clock tick time, so put
     * it in its own cacheline separated from the fields above which
     * will also be accessed at each tick.
     */
    atomic_long_t       load_avg ____cacheline_aligned;

    struct sched_rt_entity  **rt_se;
    struct rt_rq        **rt_rq;

    struct rt_bandwidth rt_bandwidth;

    struct rcu_head     rcu;
    struct list_head    list;

    struct task_group   *parent;
    struct list_head    siblings;
    struct list_head    children;

    struct autogroup    *autogroup;

    struct cfs_bandwidth    cfs_bandwidth;
};

/* CFS-related fields in a runqueue */
struct cfs_rq {
    struct load_weight  load;
    unsigned long       runnable_weight;
    unsigned int        nr_running;
    unsigned int        h_nr_running;      /* SCHED_{NORMAL,BATCH,IDLE} */
    unsigned int        idle_h_nr_running; /* SCHED_IDLE */

    u64         exec_clock;
    u64         min_vruntime;
    u64         min_vruntime_copy;

    struct rb_root_cached   tasks_timeline;

    /*
     * 'curr' points to currently running entity on this cfs_rq.
     * It is set to NULL otherwise (i.e when none are currently running).
     */
    struct sched_entity *curr;
    struct sched_entity *next;
    struct sched_entity *last;
    struct sched_entity *skip;

    unsigned int        nr_spread_over;

    /*
     * CFS load tracking
     */
    struct sched_avg    avg;
    u64         load_last_update_time_copy;
    struct {
        raw_spinlock_t  lock ____cacheline_aligned;
        int     nr;
        unsigned long   load_avg;
        unsigned long   util_avg;
        unsigned long   runnable_sum;
    } removed;

    unsigned long       tg_load_avg_contrib;
    long            propagate;
    long            prop_runnable_sum;

    /*
     *   h_load = weight * f(tg)
     *
     * Where f(tg) is the recursive weight fraction assigned to
     * this group.
     */
    unsigned long       h_load;
    u64         last_h_load_update;
    struct sched_entity *h_load_next;

    struct rq       *rq;    /* CPU runqueue to which this cfs_rq is attached */

    /*
     * leaf cfs_rqs are those that hold tasks (lowest schedulable entity in
     * a hierarchy). Non-leaf lrqs hold other higher schedulable entities
     * (like users, containers etc.)
     *
     * leaf_cfs_rq_list ties together list of leaf cfs_rq's in a CPU.
     * This list is used during load balance.
     */
    int         on_list;
    struct list_head    leaf_cfs_rq_list;
    struct task_group   *tg;    /* group that "owns" this runqueue */

    int         runtime_enabled;
    s64         runtime_remaining;

    u64         throttled_clock;
    u64         throttled_clock_task;
    u64         throttled_clock_task_time;
    int         throttled;
    int         throttle_count;
    struct list_head    throttled_list;
};




static int trace_enqueue(u32 tgid, u32 pid)
{
    u64 ts = bpf_ktime_get_ns();
    runqueue_start.update(&pid, &ts);
    return 0;
}

int trace_wake_up_new_task(struct pt_regs *ctx, struct task_struct *p)
{
    return trace_enqueue(p->tgid, p->pid);
}

int trace_ttwu_do_wakeup(struct pt_regs *ctx, struct rq *rq, struct task_struct *p,
    int wake_flags)
{
    return trace_enqueue(p->tgid, p->pid);
}

// from /sys/kernel/debug/tracing/events/sched/sched_switch/format
struct sched_switch_arg {
    u64 __unused__;
    char prev_comm[16];
    pid_t prev_pid;
    int prev_prio;
    long prev_state;
    char next_comm[16];
    pid_t next_pid;
    int next_prio;
};

// histogram indexing
static unsigned int value_to_index2(unsigned int value) {
    unsigned int index = 460;
    if (value < 100) {
        // 0-99 => [0..100)
        // 0 => 0
        // 99 => 99
        index = value;
    } else if (value < 1000) {
        // 100-999 => [100..190)
        // 100 => 100
        // 999 => 189
        index = 90 + value / 10;
    } else if (value < 10000) {
        // 1_000-9_999 => [190..280)
        // 1000 => 190
        // 9999 => 279
        index = 180 + value / 100;
    } else if (value < 100000) {
        // 10_000-99_999 => [280..370)
        // 10000 => 280
        // 99999 => 369
        index = 270 + value / 1000;
    } else if (value < 1000000) {
        // 100_000-999_999 => [370..460)
        // 100000 => 370
        // 999999 => 459
        index = 360 + value / 10000;
    } else {
        index = 460;
    }
    return index;
}

int trace_run(struct pt_regs *ctx, struct task_struct *prev)
{
    // handle involuntary context switch
    if (prev->state == TASK_RUNNING) {
        u32 tgid = prev->tgid;
        u32 pid = prev->pid;
        u64 ts = bpf_ktime_get_ns();
        runqueue_start.update(&pid, &ts);
    }

    // get tgid and pid
    u32 tgid = bpf_get_current_pid_tgid() >> 32;
    u32 pid = bpf_get_current_pid_tgid();

    // lookup start time
    u64 *tsp = runqueue_start.lookup(&pid);

    // skip events with unknown start
    if (tsp == 0) {
        return 0;
    }

    // calculate latency in microseconds
    u64 delta = (bpf_ktime_get_ns() - *tsp) / 1000;

    // calculate index and increment histogram
    unsigned int index = value_to_index2(delta);
    runqueue_latency.increment(index);

    // clear the start time
    runqueue_start.delete(&pid);
    return 0;
}

int trace_throttle(struct pt_regs *ctx, struct cfs_rq *cfs_rq)
{
    // key is the id of the kernelfs_node for the cgroup being throttled
    int id = cfs_rq->tg->css.cgroup->kn->id;
    u64 ts = bpf_ktime_get_ns();
    throttle_start.update(&id, &ts);
    return 0;
};

int trace_unthrottle(struct pt_regs *ctx, struct cfs_rq *cfs_rq)
{
    // key is the id of the kernelfs_node for the cgroup being unthrottled
    int id = cfs_rq->tg->css.cgroup->kn->id;
    u64 *tsp;
    const u64 microsecond = 1000;
    
    tsp = throttle_start.lookup(&id);
    if (tsp == 0) {
        // missed throttle, skip
        return 0;
    }
    int delta = bpf_ktime_get_ns() - *tsp;
    delta /= microsecond;
    int index = value_to_index2(delta);
    cfs_throttle.increment(index);
    throttle_start.delete(&id);
    return 0;
};
