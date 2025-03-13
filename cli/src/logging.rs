// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use console::style;
use tracing_subscriber::Layer;

// Custom layer for cliclack logging
pub struct CliclackLayer;

impl<S> Layer<S> for CliclackLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let level = event.metadata().level();
        let mut message = String::new();

        // Extract message from event
        struct MessageVisitor<'a>(&'a mut String);

        impl tracing::field::Visit for MessageVisitor<'_> {
            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                if field.name() == "message" {
                    *self.0 = value.to_string();
                }
            }

            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    *self.0 = format!("{:?}", value);
                }
            }
        }

        event.record(&mut MessageVisitor(&mut message));

        // Format message with level-specific styling
        let formatted_message = match *level {
            tracing::Level::ERROR => style(&message).red().to_string(),
            tracing::Level::WARN => style(&message).yellow().to_string(),
            tracing::Level::INFO => style(&message).cyan().to_string(),
            tracing::Level::DEBUG | tracing::Level::TRACE => style(&message).dim().to_string(),
        };

        // Route to appropriate cliclack function with formatted message
        match *level {
            tracing::Level::ERROR => cliclack::log::error(&formatted_message).ok(),
            tracing::Level::WARN => cliclack::log::warning(&formatted_message).ok(),
            tracing::Level::INFO => cliclack::log::info(&formatted_message).ok(),
            //tracing::Level::DEBUG | tracing::Level::TRACE => cliclack::log::remark(&formatted_message).ok(),
            _ => Some(()),
        };
    }
}
