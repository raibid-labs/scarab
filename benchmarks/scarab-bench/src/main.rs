use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::{self, Color, Print, Stylize},
    terminal::{self, Clear, ClearType},
;
use ratatui::
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Chart, Dataset, Gauge, Paragraph, Row, Table, Axis},
    Terminal,
;
use std::io::{self, Write};
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(author, version, about = "Scarab Terminal Benchmark Suite")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Throughput test: Dump text to stdout as fast as possible
    Flood {
        /// Size in MB to dump
        #[arg(short, long, default_value = "100")]
        size_mb: usize,
    },
    /// Latency/FPS test: Render complex TUI updates
    Tui {
        /// Duration in seconds
        #[arg(short, long, default_value = "10")]
        duration: u64,
    },
    /// Cursor movement test: Random jumps and writes
    Cursor {
        /// Number of iterations
        #[arg(short, long, default_value = "10000")]
        count: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Flood { size_mb } => run_flood(size_mb),
        Commands::Tui { duration } => run_tui(duration),
        Commands::Cursor { count } => run_cursor(count),
    }
}

fn run_flood(size_mb: usize) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let line = "The quick brown fox jumps over the lazy dog. 0123456789 !@#$%^&*()_+\n";
    let line_len = line.len();
    let total_bytes = size_mb * 1024 * 1024;
    let iterations = total_bytes / line_len;

    let start = Instant::now();
    for _ in 0..iterations {
        handle.write_all(line.as_bytes())?;
    }
    let duration = start.elapsed();

    let mb_per_sec = (size_mb as f64) / duration.as_secs_f64();
    
    // We print the result to stderr so it doesn't interfere with piping if needed
    eprintln!("Flood Benchmark:");
    eprintln!("  Size: {} MB", size_mb);
    eprintln!("  Time: {:.4}s", duration.as_secs_f64());
    eprintln!("  Speed: {:.2} MB/s", mb_per_sec);

    Ok(())
}

fn run_cursor(count: usize) -> Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    let (cols, rows) = terminal::size()?;
    let mut rng = rand::thread_rng();
    use rand::Rng;

    let start = Instant::now();
    for i in 0..count {
        let x = rng.gen_range(0..cols);
        let y = rng.gen_range(0..rows);
        let color = match i % 4 {
            0 => Color::Red,
            1 => Color::Green,
            2 => Color::Blue,
            _ => Color::Yellow,
        };

        execute!(
            stdout,
            cursor::MoveTo(x, y),
            style::SetForegroundColor(color),
            Print("█"),
            style::ResetColor
        )?;
    }
    let duration = start.elapsed();

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;

    let ops_per_sec = (count as f64) / duration.as_secs_f64();
    eprintln!("Cursor Benchmark:");
    eprintln!("  Ops: {}", count);
    eprintln!("  Time: {:.4}s", duration.as_secs_f64());
    eprintln!("  Speed: {:.2} ops/s", ops_per_sec);

    Ok(())
}

fn run_tui(duration_secs: u64) -> Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let start = Instant::now();
    let end_time = start + Duration::from_secs(duration_secs);
    let mut frame_count = 0;

    let mut data: Vec<(f64, f64)> = vec![];
    let mut x = 0.0;

    while Instant::now() < end_time {
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }

        // Update simulation data
        x += 0.1;
        data.push((x, x.sin()));
        if data.len() > 100 {
            data.remove(0);
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(10),
                    Constraint::Percentage(40),
                    Constraint::Percentage(50),
                ])
                .split(f.size());

            // Gauge
            let elapsed = start.elapsed().as_secs_f64();
            let progress = (elapsed / duration_secs as f64).min(1.0);
            let gauge = Gauge::default()
                .block(Block::default().title("Benchmark Progress").borders(Borders::ALL))
                .gauge_style(Style::default().fg(ratatui::style::Color::Magenta).bg(ratatui::style::Color::Black).add_modifier(Modifier::ITALIC))
                .ratio(progress);
            f.render_widget(gauge, chunks[0]);

            // Chart
            let datasets = vec![Dataset::default()
                .name("Sine Wave")
                .marker(ratatui::symbols::Marker::Braille)
                .style(Style::default().fg(ratatui::style::Color::Cyan))
                .data(&data)];
            let chart = Chart::new(datasets)
                .block(Block::default().title("CPU Load Simulation").borders(Borders::ALL))
                .x_axis(Axis::default().bounds([x - 10.0, x]))
                .y_axis(Axis::default().bounds([-1.0, 1.0]));
            f.render_widget(chart, chunks[1]);

            // Table
            let rows = (0..20).map(|i| {
                Row::new(vec![
                    format!("Process {}", i),
                    format!("{} PID", 1000 + i),
                    format!("{:.1}%", (i as f64 * x).cos().abs() * 100.0),
                    "Running".to_string(),
                ])
            });
            let table = Table::new(rows, [
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ])
            .header(Row::new(vec!["Name", "PID", "CPU", "Status"]).style(Style::default().fg(ratatui::style::Color::Yellow)))
            .block(Block::default().title("Process List").borders(Borders::ALL));
            f.render_widget(table, chunks[2]);
        })?;

        frame_count += 1;
    }

    terminal::disable_raw_mode()?;
    execute!(io::stdout(), terminal::LeaveAlternateScreen)?;

    let duration = start.elapsed();
    let fps = frame_count as f64 / duration.as_secs_f64();

    eprintln!("TUI Benchmark:");
    eprintln!("  Frames: {}", frame_count);
    eprintln!("  Time: {:.4}s", duration.as_secs_f64());
    eprintln!("  FPS: {:.2}", fps);

    Ok(())
}
