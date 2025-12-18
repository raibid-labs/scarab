//! Dashboard pane type for data visualization
//!
//! A special pane type that renders dashboards instead of terminal output.
//! Useful for system monitors, build progress, git status, and other visualizations.

use bevy::prelude::*;
use std::time::Duration;

/// Plugin for dashboard panes
pub struct DashboardPlugin;

impl Plugin for DashboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DashboardUpdateEvent>()
            .add_systems(
                Update,
                (
                    update_dashboard_system,
                    render_dashboard_widgets_system,
                    update_system_monitor_dashboard_system,
                ),
            );
    }
}

/// Event to trigger dashboard updates
#[derive(Event, Clone)]
pub struct DashboardUpdateEvent {
    pub dashboard_id: String,
}

/// Dashboard state for a pane
#[derive(Component, Clone)]
pub struct DashboardState {
    /// Unique identifier for this dashboard
    pub id: String,
    /// List of widgets to display
    pub widgets: Vec<DashboardWidget>,
    /// Layout mode
    pub layout: DashboardLayout,
    /// Refresh rate for automatic updates
    pub refresh_rate: Duration,
    /// Time since last update
    pub time_since_update: Duration,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            widgets: Vec::new(),
            layout: DashboardLayout::Vertical,
            refresh_rate: Duration::from_secs(1),
            time_since_update: Duration::ZERO,
        }
    }
}

impl DashboardState {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ..Default::default()
        }
    }

    pub fn with_widgets(mut self, widgets: Vec<DashboardWidget>) -> Self {
        self.widgets = widgets;
        self
    }

    pub fn with_layout(mut self, layout: DashboardLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_refresh_rate(mut self, rate: Duration) -> Self {
        self.refresh_rate = rate;
        self
    }
}

/// Layout mode for dashboard widgets
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DashboardLayout {
    /// Widgets stacked vertically
    Vertical,
    /// Widgets arranged horizontally
    Horizontal,
    /// Grid layout (2x2, 3x3, etc.)
    Grid { columns: usize },
    /// Custom layout with explicit positions
    Custom,
}

/// A dashboard widget
#[derive(Clone, Debug)]
pub enum DashboardWidget {
    /// Line chart with time series data
    LineChart {
        id: String,
        label: String,
        data: Vec<f32>,
        color: Color,
        max_points: usize,
        min_value: Option<f32>,
        max_value: Option<f32>,
    },
    /// Bar chart with labeled data
    BarChart {
        id: String,
        label: String,
        data: Vec<(String, f32)>,
        color: Color,
    },
    /// Gauge/meter for single values
    Gauge {
        id: String,
        label: String,
        value: f32,
        max: f32,
        color: Color,
        show_percentage: bool,
    },
    /// Text display
    Text {
        id: String,
        content: String,
        style: TextDisplayStyle,
    },
    /// Table with rows and columns
    Table {
        id: String,
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        max_rows: Option<usize>,
    },
}

impl DashboardWidget {
    pub fn id(&self) -> &str {
        match self {
            Self::LineChart { id, .. }
            | Self::BarChart { id, .. }
            | Self::Gauge { id, .. }
            | Self::Text { id, .. }
            | Self::Table { id, .. } => id,
        }
    }

    /// Update widget data (for line charts)
    pub fn push_data_point(&mut self, value: f32) {
        if let Self::LineChart { data, max_points, .. } = self {
            data.push(value);
            if data.len() > *max_points {
                data.remove(0);
            }
        }
    }

    /// Update widget value (for gauges)
    pub fn update_value(&mut self, new_value: f32) {
        if let Self::Gauge { value, .. } = self {
            *value = new_value;
        }
    }
}

/// Style for text widgets
#[derive(Clone, Debug, PartialEq)]
pub enum TextDisplayStyle {
    Normal,
    Title,
    Monospace,
    Highlighted,
}

/// Marker component for dashboard panes
#[derive(Component)]
pub struct DashboardPane;

/// System to update dashboard states based on refresh rate
fn update_dashboard_system(
    time: Res<Time>,
    mut query: Query<&mut DashboardState>,
    mut events: EventWriter<DashboardUpdateEvent>,
) {
    for mut dashboard in query.iter_mut() {
        dashboard.time_since_update += time.delta();

        if dashboard.time_since_update >= dashboard.refresh_rate {
            dashboard.time_since_update = Duration::ZERO;
            events.send(DashboardUpdateEvent {
                dashboard_id: dashboard.id.clone(),
            });
        }
    }
}

/// System to render dashboard widgets
fn render_dashboard_widgets_system(
    mut commands: Commands,
    query: Query<(Entity, &DashboardState), Changed<DashboardState>>,
) {
    for (entity, dashboard) in query.iter() {
        // Clear existing children
        commands.entity(entity).despawn_descendants();

        // Create UI nodes for each widget
        commands.entity(entity).with_children(|parent| {
            for widget in &dashboard.widgets {
                match widget {
                    DashboardWidget::LineChart { label, .. } => {
                        spawn_line_chart(parent, widget, label);
                    }
                    DashboardWidget::BarChart { label, .. } => {
                        spawn_bar_chart(parent, widget, label);
                    }
                    DashboardWidget::Gauge { label, .. } => {
                        spawn_gauge(parent, widget, label);
                    }
                    DashboardWidget::Text { content, .. } => {
                        spawn_text_widget(parent, widget, content);
                    }
                    DashboardWidget::Table { .. } => {
                        spawn_table_widget(parent, widget);
                    }
                }
            }
        });
    }
}

/// Spawn a line chart widget
fn spawn_line_chart(parent: &mut ChildBuilder, _widget: &DashboardWidget, label: &str) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(150.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
    ))
    .with_children(|chart_parent| {
        // Title
        chart_parent.spawn((
            Text::new(label),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));

        // Chart area (simplified - would use actual rendering in production)
        chart_parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                margin: UiRect::top(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.9)),
        ));
    });
}

/// Spawn a bar chart widget
fn spawn_bar_chart(parent: &mut ChildBuilder, _widget: &DashboardWidget, label: &str) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(200.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
    ))
    .with_children(|chart_parent| {
        // Title
        chart_parent.spawn((
            Text::new(label),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));

        // Bars area
        chart_parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                margin: UiRect::top(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.9)),
        ));
    });
}

/// Spawn a gauge widget
fn spawn_gauge(parent: &mut ChildBuilder, widget: &DashboardWidget, label: &str) {
    if let DashboardWidget::Gauge { value, max, color, show_percentage, .. } = widget {
        let percentage = (*value / *max) * 100.0;

        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(80.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
        ))
        .with_children(|gauge_parent| {
            // Label
            gauge_parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Progress bar background
            gauge_parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(30.0),
                    margin: UiRect::top(Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
            ))
            .with_children(|bar_parent| {
                // Progress fill
                bar_parent.spawn((
                    Node {
                        width: Val::Percent(percentage),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(*color),
                ));
            });

            // Value text
            if *show_percentage {
                gauge_parent.spawn((
                    Text::new(format!("{:.1}%", percentage)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::top(Val::Px(3.0)),
                        ..default()
                    },
                ));
            }
        });
    }
}

/// Spawn a text widget
fn spawn_text_widget(parent: &mut ChildBuilder, _widget: &DashboardWidget, content: &str) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
    ))
    .with_children(|text_parent| {
        text_parent.spawn((
            Text::new(content),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

/// Spawn a table widget
fn spawn_table_widget(parent: &mut ChildBuilder, widget: &DashboardWidget) {
    if let DashboardWidget::Table { headers, rows, .. } = widget {
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
        ))
        .with_children(|table_parent| {
            // Headers
            table_parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    padding: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
            ))
            .with_children(|header_parent| {
                for header in headers {
                    header_parent.spawn((
                        Text::new(header),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            padding: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                    ));
                }
            });

            // Rows
            for row in rows {
                table_parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                ))
                .with_children(|row_parent| {
                    for cell in row {
                        row_parent.spawn((
                            Text::new(cell),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.9, 0.9, 0.9, 1.0)),
                            Node {
                                padding: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                        ));
                    }
                });
            }
        });
    }
}

/// System to update system monitor dashboard
/// This is an example showing how to populate dashboard data
fn update_system_monitor_dashboard_system(
    time: Res<Time>,
    mut query: Query<&mut DashboardState>,
    mut events: EventReader<DashboardUpdateEvent>,
) {
    for event in events.read() {
        if event.dashboard_id == "system_monitor" {
            // Find the dashboard
            for mut dashboard in query.iter_mut() {
                if dashboard.id == "system_monitor" {
                    // Use time to generate fake data (sine wave pattern)
                    let t = time.elapsed_secs();

                    // Update CPU widget
                    if let Some(widget) = dashboard.widgets.iter_mut().find(|w| w.id() == "cpu") {
                        // Simulate CPU usage with sine wave (25-75%)
                        let fake_cpu = 50.0 + 25.0 * (t * 0.5).sin();
                        widget.push_data_point(fake_cpu);
                    }

                    // Update memory widget
                    if let Some(widget) = dashboard.widgets.iter_mut().find(|w| w.id() == "memory") {
                        // Simulate memory usage with slower sine wave (50-80%)
                        let fake_mem = 65.0 + 15.0 * (t * 0.2).sin();
                        widget.update_value(fake_mem);
                    }
                }
            }
        }
    }
}

/// Helper to create a system monitor dashboard
pub fn create_system_monitor_dashboard() -> DashboardState {
    DashboardState::new("system_monitor")
        .with_refresh_rate(Duration::from_secs(2))
        .with_layout(DashboardLayout::Vertical)
        .with_widgets(vec![
            DashboardWidget::LineChart {
                id: "cpu".to_string(),
                label: "CPU Usage".to_string(),
                data: vec![0.0; 60],
                color: Color::srgb(0.3, 0.8, 0.3),
                max_points: 60,
                min_value: Some(0.0),
                max_value: Some(100.0),
            },
            DashboardWidget::Gauge {
                id: "memory".to_string(),
                label: "Memory".to_string(),
                value: 65.0,
                max: 100.0,
                color: Color::srgb(0.8, 0.3, 0.3),
                show_percentage: true,
            },
            DashboardWidget::Table {
                id: "processes".to_string(),
                headers: vec!["PID".to_string(), "Name".to_string(), "CPU%".to_string()],
                rows: vec![
                    vec!["1234".to_string(), "scarab".to_string(), "12.5".to_string()],
                    vec!["5678".to_string(), "nvim".to_string(), "5.2".to_string()],
                ],
                max_rows: Some(10),
            },
        ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_state_builder() {
        let dashboard = DashboardState::new("test")
            .with_refresh_rate(Duration::from_secs(5))
            .with_layout(DashboardLayout::Grid { columns: 2 });

        assert_eq!(dashboard.id, "test");
        assert_eq!(dashboard.refresh_rate, Duration::from_secs(5));
        assert_eq!(dashboard.layout, DashboardLayout::Grid { columns: 2 });
    }

    #[test]
    fn test_widget_id() {
        let widget = DashboardWidget::Gauge {
            id: "test_gauge".to_string(),
            label: "Test".to_string(),
            value: 50.0,
            max: 100.0,
            color: Color::WHITE,
            show_percentage: true,
        };

        assert_eq!(widget.id(), "test_gauge");
    }

    #[test]
    fn test_push_data_point() {
        let mut widget = DashboardWidget::LineChart {
            id: "test".to_string(),
            label: "Test".to_string(),
            data: vec![1.0, 2.0, 3.0],
            color: Color::WHITE,
            max_points: 3,
            min_value: None,
            max_value: None,
        };

        widget.push_data_point(4.0);

        if let DashboardWidget::LineChart { data, .. } = widget {
            assert_eq!(data.len(), 3);
            assert_eq!(data[0], 2.0); // First element removed
            assert_eq!(data[2], 4.0); // New element added
        }
    }

    #[test]
    fn test_update_value() {
        let mut widget = DashboardWidget::Gauge {
            id: "test".to_string(),
            label: "Test".to_string(),
            value: 50.0,
            max: 100.0,
            color: Color::WHITE,
            show_percentage: true,
        };

        widget.update_value(75.0);

        if let DashboardWidget::Gauge { value, .. } = widget {
            assert_eq!(value, 75.0);
        }
    }
}
