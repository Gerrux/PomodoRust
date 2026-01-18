//! Animation utilities and state management

use std::time::Instant;

/// Easing functions for animations (CSS cubic-bezier compatible)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    /// CSS ease (0.25, 0.1, 0.25, 1.0)
    Ease,
    /// Smooth deceleration
    Decelerate,
    /// Smooth acceleration
    Accelerate,
    /// Overshoot and settle
    Spring,
    /// Bounce at end
    Bounce,
}

impl Easing {
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t * t,
            Easing::EaseOut => 1.0 - (1.0 - t).powi(3),
            Easing::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Easing::Ease => cubic_bezier(0.25, 0.1, 0.25, 1.0, t),
            Easing::Decelerate => cubic_bezier(0.0, 0.0, 0.2, 1.0, t),
            Easing::Accelerate => cubic_bezier(0.4, 0.0, 1.0, 1.0, t),
            Easing::Spring => {
                let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            }
            Easing::Bounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                let mut t = t;
                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    t -= 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    t -= 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    t -= 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }
        }
    }
}

/// Approximate cubic-bezier curve
fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, t: f32) -> f32 {
    // Newton-Raphson iteration to find t for x
    let mut guess = t;
    for _ in 0..8 {
        let x = bezier_sample(x1, x2, guess) - t;
        if x.abs() < 0.001 {
            break;
        }
        let dx = bezier_derivative(x1, x2, guess);
        if dx.abs() < 0.0001 {
            break;
        }
        guess -= x / dx;
    }
    bezier_sample(y1, y2, guess.clamp(0.0, 1.0))
}

fn bezier_sample(p1: f32, p2: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3
}

fn bezier_derivative(p1: f32, p2: f32, t: f32) -> f32 {
    let mt = 1.0 - t;
    3.0 * mt * mt * p1 + 6.0 * mt * t * (p2 - p1) + 3.0 * t * t * (1.0 - p2)
}

/// A single animated value with configurable easing
#[derive(Debug, Clone)]
pub struct AnimatedValue {
    start: f32,
    end: f32,
    current: f32,
    start_time: Option<Instant>,
    duration: f32,
    easing: Easing,
}

impl AnimatedValue {
    pub fn new(value: f32) -> Self {
        Self {
            start: value,
            end: value,
            current: value,
            start_time: None,
            duration: 0.3,
            easing: Easing::EaseOut,
        }
    }

    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Set target value and start animation
    pub fn animate_to(&mut self, target: f32) {
        if (self.end - target).abs() < 0.001 {
            return;
        }
        self.start = self.current;
        self.end = target;
        self.start_time = Some(Instant::now());
    }

    /// Set value immediately without animation
    pub fn set(&mut self, value: f32) {
        self.start = value;
        self.end = value;
        self.current = value;
        self.start_time = None;
    }

    /// Update and return current value
    pub fn update(&mut self) -> f32 {
        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed().as_secs_f32();
            let t = (elapsed / self.duration).clamp(0.0, 1.0);
            let eased = self.easing.apply(t);
            self.current = self.start + (self.end - self.start) * eased;

            if t >= 1.0 {
                self.current = self.end;
                self.start_time = None;
            }
        }
        self.current
    }

    pub fn is_animating(&self) -> bool {
        self.start_time.is_some()
    }

    pub fn value(&self) -> f32 {
        self.current
    }

    pub fn target(&self) -> f32 {
        self.end
    }
}

/// State for hover/press animations with smooth transitions
#[derive(Debug, Clone)]
pub struct InteractionState {
    hover: AnimatedValue,
    press: AnimatedValue,
    focus: AnimatedValue,
}

impl InteractionState {
    pub fn new() -> Self {
        Self {
            hover: AnimatedValue::new(0.0)
                .with_duration(0.12)
                .with_easing(Easing::Ease),
            press: AnimatedValue::new(0.0)
                .with_duration(0.08)
                .with_easing(Easing::EaseOut),
            focus: AnimatedValue::new(0.0)
                .with_duration(0.2)
                .with_easing(Easing::EaseOut),
        }
    }

    pub fn update(&mut self, is_hovered: bool, is_pressed: bool) {
        self.hover.animate_to(if is_hovered { 1.0 } else { 0.0 });
        self.press.animate_to(if is_pressed { 1.0 } else { 0.0 });
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focus.animate_to(if focused { 1.0 } else { 0.0 });
    }

    pub fn hover_t(&mut self) -> f32 {
        self.hover.update()
    }

    pub fn press_t(&mut self) -> f32 {
        self.press.update()
    }

    pub fn focus_t(&mut self) -> f32 {
        self.focus.update()
    }

    /// Combined interaction intensity (for glow effects)
    pub fn intensity(&mut self) -> f32 {
        let h = self.hover_t();
        let p = self.press_t();
        (h + p * 0.5).min(1.0)
    }

    pub fn is_animating(&self) -> bool {
        self.hover.is_animating() || self.press.is_animating() || self.focus.is_animating()
    }
}

impl Default for InteractionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Global animation state for the app
#[derive(Debug, Clone)]
pub struct AnimationState {
    /// Timer pulse animation (0.0 to 1.0, cycles)
    timer_pulse: f32,
    /// Secondary pulse (slower)
    glow_phase: f32,
    /// Breathing animation for idle state
    breathe_phase: f32,
    /// Progress ring animation
    progress_anim: AnimatedValue,
    /// View transition
    view_transition: AnimatedValue,
    /// Last update time
    last_update: Instant,
    /// Is timer running (for conditional animations)
    timer_running: bool,
}

impl AnimationState {
    pub fn new() -> Self {
        Self {
            timer_pulse: 0.0,
            glow_phase: 0.0,
            breathe_phase: 0.0,
            progress_anim: AnimatedValue::new(0.0)
                .with_duration(0.6)
                .with_easing(Easing::Decelerate),
            view_transition: AnimatedValue::new(0.0)
                .with_duration(0.25)
                .with_easing(Easing::Ease),
            last_update: Instant::now(),
            timer_running: false,
        }
    }

    /// Update all continuous animations
    pub fn update(&mut self, timer_running: bool) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        self.timer_running = timer_running;

        // Timer pulse (1.5 second cycle when running)
        if timer_running {
            self.timer_pulse += dt / 1.5;
            if self.timer_pulse > 1.0 {
                self.timer_pulse -= 1.0;
            }
        } else {
            self.timer_pulse = (self.timer_pulse - dt * 3.0).max(0.0);
        }

        // Background glow (4 second cycle, always active but subtle)
        self.glow_phase += dt / 4.0;
        if self.glow_phase > 1.0 {
            self.glow_phase -= 1.0;
        }

        // Breathing animation (3 second cycle, only when idle)
        if !timer_running {
            self.breathe_phase += dt / 3.0;
            if self.breathe_phase > 1.0 {
                self.breathe_phase -= 1.0;
            }
        } else {
            self.breathe_phase = 0.0;
        }

        self.progress_anim.update();
        self.view_transition.update();
    }

    /// Get pulse value for timer (smooth sine wave)
    pub fn pulse_value(&self) -> f32 {
        if self.timer_running {
            // Smooth pulse when running
            let t = self.timer_pulse * std::f32::consts::TAU;
            (t.sin() * 0.5 + 0.5).powf(0.7) // Slightly sharper pulse
        } else {
            0.0
        }
    }

    /// Get breathing value for idle animations
    pub fn breathe_value(&self) -> f32 {
        let t = self.breathe_phase * std::f32::consts::TAU;
        t.sin() * 0.5 + 0.5
    }

    /// Get glow intensity (subtle ambient animation)
    pub fn glow_value(&self) -> f32 {
        let t = self.glow_phase * std::f32::consts::TAU;
        t.sin() * 0.3 + 0.7 // Range: 0.4 to 1.0
    }

    pub fn needs_repaint(&self) -> bool {
        self.timer_pulse > 0.0
            || self.breathe_phase > 0.0
            || self.progress_anim.is_animating()
            || self.view_transition.is_animating()
    }

    pub fn set_progress(&mut self, progress: f32) {
        self.progress_anim.animate_to(progress);
    }

    pub fn progress(&mut self) -> f32 {
        self.progress_anim.update()
    }

    pub fn transition_view(&mut self, forward: bool) {
        self.view_transition.set(if forward { 0.0 } else { 1.0 });
        self.view_transition
            .animate_to(if forward { 1.0 } else { 0.0 });
    }

    pub fn view_transition_t(&mut self) -> f32 {
        self.view_transition.update()
    }
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Smooth number counter animation using spring physics
#[derive(Debug, Clone)]
pub struct CounterAnimation {
    current: f32,
    target: f32,
    velocity: f32,
}

impl CounterAnimation {
    pub fn new(initial: f32) -> Self {
        Self {
            current: initial,
            target: initial,
            velocity: 0.0,
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    pub fn update(&mut self, dt: f32) -> f32 {
        // Critically damped spring
        let stiffness: f32 = 120.0;
        let damping = 2.0 * stiffness.sqrt(); // Critical damping

        let spring_force = stiffness * (self.target - self.current);
        let damping_force = -damping * self.velocity;
        let acceleration = spring_force + damping_force;

        self.velocity += acceleration * dt;
        self.current += self.velocity * dt;

        // Snap when close
        if (self.target - self.current).abs() < 0.01 && self.velocity.abs() < 0.01 {
            self.current = self.target;
            self.velocity = 0.0;
        }

        self.current
    }

    pub fn value(&self) -> f32 {
        self.current
    }

    pub fn is_animating(&self) -> bool {
        (self.target - self.current).abs() > 0.01 || self.velocity.abs() > 0.01
    }
}

/// Staggered animation helper for lists
pub struct StaggeredAnimation {
    items: Vec<AnimatedValue>,
    stagger_delay: f32,
}

impl StaggeredAnimation {
    pub fn new(count: usize, stagger_delay: f32) -> Self {
        Self {
            items: (0..count)
                .map(|_| {
                    AnimatedValue::new(0.0)
                        .with_duration(0.3)
                        .with_easing(Easing::Decelerate)
                })
                .collect(),
            stagger_delay,
        }
    }

    pub fn animate_in(&mut self) {
        let now = Instant::now();
        for (i, item) in self.items.iter_mut().enumerate() {
            let delay = i as f32 * self.stagger_delay;
            // Delay by manipulating start time
            item.start_time = Some(now - std::time::Duration::from_secs_f32(-delay));
            item.start = 0.0;
            item.end = 1.0;
        }
    }

    pub fn get(&mut self, index: usize) -> f32 {
        self.items.get_mut(index).map(|v| v.update()).unwrap_or(1.0)
    }

    pub fn is_animating(&self) -> bool {
        self.items.iter().any(|v| v.is_animating())
    }
}
