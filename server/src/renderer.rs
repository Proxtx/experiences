use crate::{config::Config, PluginRenderers};
use futures::{future::BoxFuture, FutureExt};
use raqote::{DrawTarget, Image, IntRect};
use shared::{
    timeline::types::api::{AvailablePlugins, CompressedEvent},
    types::Experience,
};
use std::{collections::HashMap, pin::Pin, sync::Arc};

pub trait PluginRenderer: Send + Sync {
    fn new() -> impl std::future::Future<Output = Self> + Send
    where
        Self: Sized;
    fn render(
        &self,
        dimensions: (i32, i32),
        event: &CompressedEvent,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<u32>, String>> + Send>>;
}

pub struct Renderer {
    plugins: HashMap<AvailablePlugins, Box<dyn PluginRenderer>>,
}

impl Renderer {
    pub async fn new() -> Renderer {
        Renderer {
            plugins: PluginRenderers::init().await.renderers,
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

    pub async fn render_events(
        &self,
        size: i32,
        events: Vec<(AvailablePlugins, &CompressedEvent)>,
    ) -> DrawTarget {
        let mut resolved_events = Vec::new();
        Renderer::resolve_events((0., 0.), &mut resolved_events, events, size as f32);
        println!("{:?}", resolved_events);

        let mut draw_target = DrawTarget::new(size, size);

        for render_request in resolved_events {
            let rendered_event = self
                .render_event(
                    &render_request.plugin,
                    render_request.event,
                    (render_request.width as i32, render_request.height as i32),
                )
                .await;
            draw_target.copy_surface(
                &rendered_event,
                IntRect {
                    min: (0, 0).into(),
                    max: (render_request.width as i32, render_request.height as i32).into(),
                },
                (render_request.x as i32, render_request.y as i32).into(),
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
        println!("Resolve events call with {} events.", events.len());
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
                let side_len = ((len as f32).sqrt().ceil()) as i32;
                let block_size = size / side_len as f32;

                //fill top/bottom side
                for po in 0..side_len {
                    println!("po{}", po);
                    let pos = po;
                    let x = block_size * pos as f32;
                    let y = 0.;
                    let y2 = size - block_size;
                    let vt = events.pop().unwrap();
                    let vt2 = events.pop().unwrap();
                    resolved_events.push(ResolvedEventRender {
                        x: transform.0 + x,
                        y: transform.1 + y,
                        width: block_size,
                        height: block_size,
                        plugin: vt.0,
                        event: vt.1,
                    });

                    resolved_events.push(ResolvedEventRender {
                        x: transform.0 + x,
                        y: transform.1 + y2,
                        width: block_size,
                        height: block_size,
                        plugin: vt2.0,
                        event: vt2.1,
                    });
                }

                //fill left/right side
                for lpi in 1..(side_len - 1) {
                    let pos = lpi - 1;
                    let x = 0.;
                    let x2 = size - block_size;
                    let y = block_size * pos as f32;
                    let vt = events.pop().unwrap();
                    let vt2 = events.pop().unwrap();
                    resolved_events.push(ResolvedEventRender {
                        x: transform.0 + x,
                        y: transform.1 + y,
                        width: block_size,
                        height: block_size,
                        plugin: vt.0,
                        event: vt.1,
                    });

                    resolved_events.push(ResolvedEventRender {
                        x: transform.0 + x2,
                        y: transform.1 + y,
                        width: block_size,
                        height: block_size,
                        plugin: vt2.0,
                        event: vt2.1,
                    });
                }

                Renderer::resolve_events(
                    (transform.0 + block_size, transform.1 + block_size),
                    resolved_events,
                    events,
                    size - 2. * block_size,
                )
            }
        };
    }
}

#[derive(Debug)]
struct ResolvedEventRender<'a> {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub plugin: AvailablePlugins,
    pub event: &'a CompressedEvent,
}
