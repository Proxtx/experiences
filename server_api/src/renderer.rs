use {
    raqote::{DrawOptions, DrawTarget, IntRect, SolidSource},
    timeline_types::api::CompressedEvent,
    timeline_types::available_plugins::AvailablePlugins,
    shared::types::Experience,
    std::{collections::HashMap, fmt::Debug},
    crate::plugin::PluginRenderer,
};

pub struct Renderer {
    plugins: HashMap<AvailablePlugins, Box<dyn PluginRenderer>>,
}

impl Renderer {
    pub fn new(renderers: HashMap<AvailablePlugins, Box<dyn PluginRenderer>>) -> Renderer {
        Renderer {
            plugins: renderers,
        }
    }

    pub async fn render_event(
        &self,
        plugin: &AvailablePlugins,
        event: &CompressedEvent,
        dimensions: (i32, i32),
    ) -> DrawTarget {
        let mut draw_target = DrawTarget::new(dimensions.0, dimensions.1);
        if let Some(renderer) = self.plugins.get(plugin) {
            match renderer.render(dimensions, event).await {
                Ok(v) => {
                    draw_target = DrawTarget::from_vec(dimensions.0, dimensions.1, v);
                }
                Err(e) => eprintln!("Error: Unable to render experience picture: {}", e),
            }
        }

        draw_target
    }

    pub async fn render_experience(&self, experience: &Experience, size: i32) -> DrawTarget {
        let mut favorites = Vec::new();
        for (plugin, events) in experience.events.iter() {
            for event in events.iter() {
                if event.favorite {
                    favorites.push((plugin.clone(), &event.event))
                }
            }
        }
        self.render_events(size, favorites).await
    }

    pub async fn render_entire_experience(&self, experience: &Experience, size: i32) -> DrawTarget {
        let mut events = experience
            .events
            .iter()
            .flat_map(|(k, v)| v.iter().map(|v| (k.clone(), &v.event)).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let mut target = DrawTarget::new(size, size);

        loop {
            let mut resolved_events = Vec::new();
            Renderer::resolve_events((0., 0.), &mut resolved_events, events.clone(), size as f32);

            target.fill_rect(
                0.,
                0.,
                size as f32,
                size as f32,
                &raqote::Source::Solid(SolidSource::from_unpremultiplied_argb(255, 0, 0, 0)),
                &DrawOptions::new(),
            );

            let mut failed_events = Vec::new();

            for render_request in resolved_events {
                let lc_dimensions = (
                    render_request.width.ceil() as i32,
                    render_request.height.ceil() as i32,
                );
                if let Some(renderer) = self.plugins.get(&render_request.plugin) {
                    if let Ok(v) = renderer.render(lc_dimensions, render_request.event).await {
                        let event_target =
                            DrawTarget::from_vec(lc_dimensions.0, lc_dimensions.1, v);
                        target.copy_surface(
                            &event_target,
                            IntRect {
                                min: (0, 0).into(),
                                max: lc_dimensions.into(),
                            },
                            (
                                render_request.x.ceil() as i32,
                                render_request.y.ceil() as i32,
                            )
                                .into(),
                        );
                        continue;
                    }
                }
                failed_events.push(render_request.event);
            }
            if !failed_events.is_empty() {
                events.retain(|(_, v)| !failed_events.contains(v));
            } else {
                break;
            };
        }

        target
    }

    pub async fn render_events(
        &self,
        size: i32,
        events: Vec<(AvailablePlugins, &CompressedEvent)>,
    ) -> DrawTarget {
        let mut resolved_events = Vec::new();
        Renderer::resolve_events((0., 0.), &mut resolved_events, events, size as f32);

        let mut draw_target = DrawTarget::new(size, size);
        draw_target.fill_rect(
            0.,
            0.,
            size as f32,
            size as f32,
            &raqote::Source::Solid(SolidSource::from_unpremultiplied_argb(255, 0, 0, 0)),
            &DrawOptions::new(),
        );

        for render_request in resolved_events {
            let rendered_event = self
                .render_event(
                    &render_request.plugin,
                    render_request.event,
                    (
                        render_request.width.ceil() as i32,
                        render_request.height.ceil() as i32,
                    ),
                )
                .await;
            draw_target.copy_surface(
                &rendered_event,
                IntRect {
                    min: (0, 0).into(),
                    max: (
                        render_request.width.ceil() as i32,
                        render_request.height.ceil() as i32,
                    )
                        .into(),
                },
                (
                    render_request.x.ceil() as i32,
                    render_request.y.ceil() as i32,
                )
                    .into(),
            );
        }

        draw_target
    }

    fn resolve_events<'b>(
        transform: (f32, f32),
        resolved_events: &mut Vec<ResolvedEventRender<'b>>,
        mut events: Vec<(AvailablePlugins, &'b CompressedEvent)>,
        size: f32,
    ) {
        match events.len() {
            0 => (),
            1 => {
                let vt = events.pop().unwrap();
                resolved_events.push(ResolvedEventRender {
                    x: transform.0,
                    y: transform.1,
                    width: size,
                    height: size,
                    plugin: vt.0,
                    event: vt.1,
                });
            }
            2 => {
                let vt1 = events.pop().unwrap();
                resolved_events.push(ResolvedEventRender {
                    x: transform.0,
                    y: transform.1,
                    width: size,
                    height: size / 2.,
                    plugin: vt1.0,
                    event: vt1.1,
                });

                let vt2 = events.pop().unwrap();
                resolved_events.push(ResolvedEventRender {
                    x: transform.0,
                    y: transform.1 + size / 2.,
                    width: size,
                    height: size / 2.,
                    plugin: vt2.0,
                    event: vt2.1,
                });
            }
            3 => {
                let vt1 = events.pop().unwrap();
                resolved_events.push(ResolvedEventRender {
                    x: transform.0,
                    y: transform.1,
                    width: size / 2.,
                    height: size / 2.,
                    plugin: vt1.0,
                    event: vt1.1,
                });

                let vt2 = events.pop().unwrap();
                resolved_events.push(ResolvedEventRender {
                    x: transform.0 + size / 2.,
                    y: transform.1,
                    width: size / 2.,
                    height: size / 2.,
                    plugin: vt2.0,
                    event: vt2.1,
                });

                let vt3 = events.pop().unwrap();
                resolved_events.push(ResolvedEventRender {
                    x: transform.0,
                    y: transform.1 + size / 2.,
                    width: size,
                    height: size / 2.,
                    plugin: vt3.0,
                    event: vt3.1,
                });
            }
            len => {
                let side_len = if (10..=12).contains(&len) {
                    3
                } else if len == 5 {
                    2
                } else {
                    ((len as f32).sqrt().ceil()) as i32
                };

                let block_size = size / side_len as f32;
                let within_critical_range = (5..=8).contains(&len);

                //fill top/bottom side
                for po in 0..side_len {
                    let pos = po;
                    let x = block_size * pos as f32;
                    let y = 0.;
                    let y2 = size - block_size;
                    let vt = events.pop().unwrap();
                    resolved_events.push(ResolvedEventRender {
                        x: transform.0 + x,
                        y: transform.1 + y,
                        width: block_size,
                        height: block_size,
                        plugin: vt.0,
                        event: vt.1,
                    });

                    if !within_critical_range {
                        let vt2 = events.pop().unwrap();
                        resolved_events.push(ResolvedEventRender {
                            x: transform.0 + x,
                            y: transform.1 + y2,
                            width: block_size,
                            height: block_size,
                            plugin: vt2.0,
                            event: vt2.1,
                        });
                    }
                }

                let side_len_modifier = if within_critical_range { 0 } else { 1 };

                //fill left/right side
                for lpi in 1..(side_len - side_len_modifier) {
                    let pos = lpi;
                    let x = 0.;
                    let x2 = size - block_size;
                    let y = block_size * pos as f32;
                    let vt = events.pop().unwrap();
                    resolved_events.push(ResolvedEventRender {
                        x: transform.0 + x,
                        y: transform.1 + y,
                        width: block_size,
                        height: block_size,
                        plugin: vt.0,
                        event: vt.1,
                    });

                    if !within_critical_range {
                        let vt2 = events.pop().unwrap();
                        resolved_events.push(ResolvedEventRender {
                            x: transform.0 + x2,
                            y: transform.1 + y,
                            width: block_size,
                            height: block_size,
                            plugin: vt2.0,
                            event: vt2.1,
                        });
                    }
                }

                let new_size_modifier = if within_critical_range { 1. } else { 2. };

                Renderer::resolve_events(
                    (transform.0 + block_size, transform.1 + block_size),
                    resolved_events,
                    events,
                    size - new_size_modifier * block_size,
                )
            }
        };
    }
}

//#[derive(Debug)]
struct ResolvedEventRender<'a> {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub plugin: AvailablePlugins,
    pub event: &'a CompressedEvent,
}

impl Debug for ResolvedEventRender<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "X: {} Y: {} Width: {} Height: {}",
            self.x, self.y, self.width, self.height
        )
    }
}
