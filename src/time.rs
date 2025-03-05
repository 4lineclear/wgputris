use std::ops::ControlFlow;
use std::time::{Duration, Instant};

pub const TICK_RATE: usize = 120;
pub static TICK_DURATION: Duration = Duration::from_nanos(8333333);

#[derive(Debug, Clone, Copy)]
pub struct TimeAction {
    pub render: bool,
    pub ticks: u32,
    pub elapsed: Duration,
    pub sleep: Instant,
    pub now: Instant,
}

impl TimeAction {
    pub fn new(render: bool, ticks: u32, elapsed: Duration, sleep: Instant, now: Instant) -> Self {
        Self {
            render,
            ticks,
            elapsed,
            sleep,
            now,
        }
    }
}

pub struct Timer {
    render_rate: usize,
    render_duration: Duration,
    elapsed: Duration,
    now: Instant,
    start: Instant,
    next_tick: Instant,
    next_render: Instant,
    ticks: u32,
    renders: u32,
    tick_calls: u32,
    total_sleep_time: Duration,
}

impl Timer {
    pub fn new(render_rate: usize) -> Self {
        let now = Instant::now();
        let render_duration = Duration::from_secs_f64(1.0 / render_rate as f64);
        Self {
            render_duration,
            render_rate,
            elapsed: Duration::default(),
            now,
            start: now,
            next_tick: now + TICK_DURATION,
            next_render: now + render_duration,
            ticks: 0,
            renders: 0,
            tick_calls: 0,
            total_sleep_time: Duration::default(),
        }
    }

    pub fn tick(&mut self) -> TimeAction {
        let now = Instant::now();
        let elapsed = now - self.now;
        self.elapsed += elapsed;
        self.now = now;

        // TODO: move sleep to after checking ticks & render
        let (render, ticks) = self.tick_count(now);
        let sleep = self.next_tick.min(self.next_render);
        if ticks != 0 {
            self.next_tick = diff_time(now, TICK_DURATION, self.next_tick);
            self.ticks += ticks;
        }
        if render {
            self.next_render = diff_time(now, self.render_duration, self.next_render);
            self.renders += 1;
        }

        self.tick_calls += 1;
        self.total_sleep_time += sleep.duration_since(now);

        TimeAction {
            render,
            ticks,
            elapsed,
            sleep,
            now,
        }
    }

    fn tick_count(&mut self, now: Instant) -> (bool, u32) {
        let mut render = false;
        let mut ticks = 0;
        while self.next_render < now {
            render = true;
            self.next_render += self.render_duration;
        }
        while self.next_tick < now {
            ticks += 1;
            self.next_tick += TICK_DURATION;
        }
        (render, ticks)
    }

    pub fn sleep_until(&self) -> Instant {
        self.next_tick.min(self.next_render)
    }

    pub fn start(&self) -> Instant {
        self.start
    }

    pub fn render_rate(&self) -> usize {
        self.render_rate
    }

    /// utility method
    pub fn tick_rate(&self) -> usize {
        TICK_RATE
    }

    pub fn tick_drift(&self) -> i32 {
        self.ticks as i32 - self.est_ticks() as i32
    }

    pub fn render_drift(&self) -> i32 {
        self.renders as i32 - self.est_renders() as i32
    }

    pub fn est_renders(&self) -> u32 {
        (self.start.elapsed().as_nanos() / self.render_duration.as_nanos()) as u32
    }

    pub fn est_ticks(&self) -> u32 {
        (self.start.elapsed().as_nanos() / TICK_DURATION.as_nanos()) as u32
    }

    pub fn ticks(&self) -> u32 {
        self.ticks
    }

    pub fn renders(&self) -> u32 {
        self.renders
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

#[inline]
pub fn diff_time(now: Instant, diff: Duration, time: Instant) -> Instant {
    now + diff - diff.saturating_sub(time.saturating_duration_since(now))
}

pub fn run<B, C, Tick, Render>(
    tick: Tick,
    render: Render,
    render_rate: usize,
) -> std::thread::JoinHandle<B>
where
    Tick: Fn(TimeAction, &Timer) -> ControlFlow<B, C> + Send + 'static,
    Render: Fn(TimeAction, &Timer) + Send + 'static,
    B: Send + 'static,
{
    let mut timer = Timer::new(render_rate);
    let mut sleep = timer.sleep_until();
    std::thread::spawn(move || loop {
        let now = Instant::now();
        let sleep_dur = sleep.saturating_duration_since(now);
        if !sleep_dur.is_zero() {
            std::thread::sleep(sleep_dur);
        }
        let action = timer.tick();

        // log::info!(
        //     "sleep duration {}\n\
        //      tick drift: {} diff {}\n\
        //      render drift: {} diff {}\n\
        //      elapsed: {}, actual: {}\n\
        //      avg sleep {}",
        //     now.elapsed().as_secs_f32(),
        //     timer.ticks(),
        //     timer.tick_drift(),
        //     timer.renders(),
        //     timer.render_drift(),
        //     timer.elapsed().as_millis(),
        //     timer.start().elapsed().as_millis(),
        //     (timer.total_sleep_time / timer.tick_calls).as_secs_f32()
        // );

        if action.ticks != 0 {
            match tick(action, &timer) {
                ControlFlow::Continue(_) => (),
                ControlFlow::Break(b) => break b,
            }
        }
        if action.render {
            render(action, &timer);
        }

        sleep = action.sleep;
    })
}
