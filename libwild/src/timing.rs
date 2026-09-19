//! Code for reporting how long each phase of linking takes when the --time argument is supplied.

use crate::args::CounterKind;
use crate::env;
use crate::error::AlreadyInitialised;
use crate::error::Result;
use crate::platforms::perf::CounterList;
use anyhow::Context;
use anyhow::anyhow;
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;
use tracing::field::Visit;

const PERFETTO_ENV_VAR: &str = "WILD_PERFETTO_OUT";

pub fn setup() -> Result {
    if perfetto_output_file().is_some() {
        perfetto_recorder::start().map_err(
            |_: perfetto_recorder::TracingDisabledAtBuildTime| {
                anyhow!(
                    "{PERFETTO_ENV_VAR} was set, but wild was built without --features perfetto"
                )
            },
        )?;
    }
    Ok(())
}

#[macro_export]
macro_rules! timing_guard {
    ($($args:tt)*) => {
        (tracing::info_span!($($args)*).entered(), perfetto_recorder::start_span!($($args)*))
    };
}

#[macro_export]
macro_rules! timing_phase {
    ($($args:tt)*) => {
        let _guard = $crate::timing_guard!($($args)*);
    };
}

/// More verbose timing instrumentation that by default doesn't show up in the output of --time.
/// Suitable for use from threads other than main.
#[macro_export]
macro_rules! verbose_timing_phase {
    ($($args:tt)*) => {
        perfetto_recorder::scope!($($args)*);
    };
}

struct TimingLayer {
    counters: Mutex<CounterList>,
}

struct Data {
    start: Instant,
    child_count: u32,
    attributes_string: String,
    counters: Vec<Option<CounterSnapshot>>,
}

#[derive(Default)]
pub struct ValuesFormatter {
    out: String,
}

impl ValuesFormatter {
    fn finish(mut self) -> String {
        if !self.out.is_empty() {
            self.out.push(']');
        }
        self.out
    }
}

impl Visit for ValuesFormatter {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        use std::fmt::Write;

        if self.out.is_empty() {
            write!(&mut self.out, " [").unwrap();
        } else {
            write!(&mut self.out, ", ").unwrap();
        }
        match field.name() {
            "message" => {
                write!(&mut self.out, "{value:?}").unwrap();
            }
            name => {
                write!(&mut self.out, "{name}={value:?}").unwrap();
            }
        }
    }
}

impl<S> tracing_subscriber::Layer<S> for TimingLayer
where
    S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    fn max_level_hint(&self) -> Option<tracing::level_filters::LevelFilter> {
        Some(tracing::level_filters::LevelFilter::INFO)
    }

    fn on_new_span(
        &self,
        attributes: &tracing::span::Attributes,
        id: &tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<S>,
    ) {
        if *attributes.metadata().level() > tracing::Level::INFO {
            return;
        }
        let span = ctx.span(id).expect("valid span ID");

        let mut formatted = ValuesFormatter::default();
        attributes.values().record(&mut formatted);

        span.extensions_mut().insert(Data {
            start: Instant::now(),
            counters: Vec::new(),
            child_count: 0,
            attributes_string: formatted.finish(),
        });
    }

    fn on_enter(&self, id: &tracing::span::Id, ctx: tracing_subscriber::layer::Context<S>) {
        let span = ctx.span(id).expect("valid span ID");
        if let Some(data) = span.extensions_mut().get_mut::<Data>() {
            data.start = Instant::now();
            data.counters = self.counters.lock().unwrap().read();
        }
    }

    fn on_close(&self, id: tracing::span::Id, ctx: tracing_subscriber::layer::Context<S>) {
        let span = ctx.span(&id).expect("valid span ID");
        let metadata = span.metadata();
        if *metadata.level() > tracing::Level::INFO {
            return;
        }

        let parent_child_count = span
            .parent()
            .and_then(|parent| {
                parent
                    .extensions_mut()
                    .get_mut::<Data>()
                    .map(|parent_data| {
                        parent_data.child_count += 1;
                        parent_data.child_count
                    })
            })
            .unwrap_or(0);

        if let Some(data) = span.extensions_mut().get_mut::<Data>() {
            let scope_depth = span.scope().count() - 1;
            let name = metadata.name();
            let wall = data.start.elapsed();

            let counter_values = self
                .counters
                .lock()
                .unwrap()
                .read()
                .into_iter()
                .zip(&data.counters)
                .map(|(end, start)| end?.since(start.as_ref()?))
                .collect();

            let reading = Reading {
                wall,
                counter_values,
            };

            let indent = Indent {
                scope_depth,
                child_count: data.child_count,
                parent_child_count,
            };

            println!("{indent}{reading} {name}{}", data.attributes_string);
        }
    }
}

pub(crate) fn init_tracing(opts: &[CounterKind]) -> Result<(), AlreadyInitialised> {
    use tracing_subscriber::prelude::*;

    // Create inherited counters before we spawn worker threads, otherwise the work done by those
    // threads won't be counted.
    let layer = TimingLayer {
        counters: Mutex::new(CounterList::from_kinds(opts)),
    };

    let subscriber = tracing_subscriber::Registry::default().with(layer);
    tracing::subscriber::set_global_default(subscriber).map_err(|_| AlreadyInitialised)
}

pub(crate) struct CounterSnapshot {
    pub(crate) count: u64,
    pub(crate) time_enabled: u64,
    pub(crate) time_running: u64,
}

#[derive(Debug, PartialEq, Eq)]
struct CounterValue {
    count: u64,
    estimated: bool,
}

impl CounterSnapshot {
    fn since(&self, start: &Self) -> Option<CounterValue> {
        let count = self.count.checked_sub(start.count)?;
        let enabled = self.time_enabled.checked_sub(start.time_enabled)?;
        let running = self.time_running.checked_sub(start.time_running)?;
        if running == 0 || running > enabled {
            return None;
        }
        let scaled = u128::from(count) * u128::from(enabled) / u128::from(running);
        Some(CounterValue {
            count: u64::try_from(scaled).ok()?,
            estimated: running < enabled,
        })
    }
}

struct Reading {
    wall: Duration,
    counter_values: Vec<Option<CounterValue>>,
}

struct Indent {
    scope_depth: usize,
    parent_child_count: u32,
    child_count: u32,
}

impl Display for Indent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.scope_depth == 0 {
            write!(f, "└─")?;
            return Ok(());
        }
        for _ in 0..self.scope_depth - 1 {
            write!(f, "│ ")?;
        }
        if self.parent_child_count >= 2 {
            write!(f, "├─")?;
        } else {
            write!(f, "┌─")?;
        }
        if self.child_count > 0 {
            write!(f, "┴─")?;
        } else {
            write!(f, "──")?;
        }
        Ok(())
    }
}

impl Display for Reading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ms = self.wall.as_secs_f64() * 1000.0;
        write!(f, "{ms:>8.2}")?;

        if !self.counter_values.is_empty() {
            write!(f, " (")?;
            let mut first = true;
            for value in &self.counter_values {
                if first {
                    first = false;
                } else {
                    write!(f, ", ")?;
                }
                match value {
                    Some(value) => {
                        write!(f, "{}", value.count)?;
                        if value.estimated {
                            write!(f, " (estimated)")?;
                        }
                    }
                    None => write!(f, "unavailable")?,
                }
            }
            write!(f, ")")?;
        }

        Ok(())
    }
}

fn perfetto_output_file() -> Option<PathBuf> {
    env::var(PERFETTO_ENV_VAR).ok().map(PathBuf::from)
}

pub(crate) fn finalise_perfetto_trace() -> Result {
    let Some(path) = perfetto_output_file() else {
        return Ok(());
    };

    let mut trace = perfetto_recorder::TraceBuilder::new()?;

    trace.process_thread_data(&perfetto_recorder::ThreadTraceData::take_current_thread());
    let trace = Mutex::new(trace);

    rayon::in_place_scope(|scope| {
        scope.spawn_broadcast(|_scope, _ctx| {
            trace
                .lock()
                .unwrap()
                .process_thread_data(&perfetto_recorder::ThreadTraceData::take_current_thread());
        });
    });

    trace
        .into_inner()
        .unwrap()
        .write_to_file(&path)
        .with_context(|| format!("Failed to write perfetto trace to `{}`", path.display()))?;

    Ok(())
}
