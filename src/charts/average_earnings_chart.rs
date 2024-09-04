// Furtherance - Track your time without being tracked
// Copyright (C) 2022  Ricky Kresslein <rk@lakoliu.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{
    app::Message,
    constants::{CHART_COLOR, CHART_HEIGHT},
    models::fur_task::FurTask,
};
use chrono::NaiveDate;
use iced::{Element, Length};
use plotters::prelude::*;
use plotters_backend::DrawingBackend;
use plotters_iced::{plotters_backend, Chart, ChartWidget};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct AverageEarningsChart {
    tasks: Vec<FurTask>,
}

impl AverageEarningsChart {
    pub fn new(tasks: Vec<FurTask>) -> Self {
        Self { tasks }
    }

    pub fn view(&self) -> Element<Message> {
        let chart = ChartWidget::new(self)
            .width(Length::Fill)
            .height(Length::Fixed(if self.tasks.is_empty() {
                0.0
            } else {
                CHART_HEIGHT
            }));

        chart.into()
    }

    pub fn earnings_per_day(&self) -> BTreeMap<NaiveDate, f32> {
        self.tasks
            .iter()
            .fold(BTreeMap::new(), |mut accumulator, task| {
                let date = task.start_time.date_naive();
                let entry = accumulator.entry(date).or_insert((0.0, 0));
                entry.0 += task.total_earnings();
                entry.1 += 1;
                accumulator
            })
            .into_iter()
            .map(|(date, (total_time, count))| (date, total_time / count as f32))
            .collect()
    }
}

impl Chart<Message> for AverageEarningsChart {
    type State = ();
    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut chart: ChartBuilder<DB>) {
        let date_earned: BTreeMap<NaiveDate, f32> = self.earnings_per_day();
        let min_earned = date_earned
            .values()
            .copied()
            .max_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        let min_minus_five_percent = min_earned as f32 - (min_earned as f32 * 0.05);
        let max_earned = date_earned
            .values()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        if date_earned.len() > 1 {
            if let Some(first_date) = date_earned.first_key_value() {
                if let Some(last_date) = date_earned.last_key_value() {
                    let mut chart = chart
                        .margin(30)
                        .caption(
                            "Average Earnings Per Task",
                            ("sans-serif", 15).into_font().color(&light_dark_color()),
                        )
                        .x_label_area_size(30)
                        .y_label_area_size(30)
                        .build_cartesian_2d(
                            *first_date.0..*last_date.0,
                            min_minus_five_percent..max_earned,
                        )
                        .unwrap();

                    chart
                        .configure_mesh()
                        .label_style(&light_dark_color())
                        .x_label_style(("sans-serif", 12).into_font().color(&light_dark_color()))
                        .y_label_style(
                            ("sans-serif", 12)
                                .into_font()
                                .color(&light_dark_color())
                                .transform(FontTransform::Rotate90),
                        )
                        .y_label_formatter(&|y| format!("${:.2}", y))
                        .axis_style(ShapeStyle::from(light_dark_color()).stroke_width(1))
                        .draw()
                        .unwrap();

                    chart
                        .draw_series(LineSeries::new(
                            date_earned.iter().map(|(d, t)| (*d, *t)),
                            CHART_COLOR.filled(),
                        ))
                        .unwrap();
                }
            }
        }
    }
}

fn light_dark_color() -> RGBColor {
    match dark_light::detect() {
        dark_light::Mode::Light | dark_light::Mode::Default => BLACK,
        dark_light::Mode::Dark => WHITE,
    }
}

fn seconds_to_hms(total_seconds: &i64) -> String {
    let h = total_seconds / 3600;
    let m = total_seconds % 3600 / 60;
    let s = total_seconds % 60;
    format!("{}:{:02}:{:02}", h, m, s)
}