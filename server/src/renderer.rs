use {
    crate::PluginRenderers,
    raqote::{DrawOptions, DrawTarget, IntRect, SolidSource},
    shared::{
        timeline::types::api::{AvailablePlugins, CompressedEvent},
        types::Experience,
    },
    std::{collections::HashMap, fmt::Debug, pin::Pin},
};

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

impl<'a> Debug for ResolvedEventRender<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "X: {} Y: {} Width: {} Height: {}",
            self.x, self.y, self.width, self.height
        )
    }
}

pub async fn render_image () {
            let data = event.data.clone();

        Box::pin(async move {
            let mut target = DrawTarget::new(dimensions.0, dimensions.1);
            let path = match serde_json::from_str::<SignedMedia>(&data) {
                Ok(v) => v,
                Err(e) => {
                    return Err(format!("Unable to read CompressedEvent: {}", e))
                }
            }.path;
            let mut img = match image::ImageReader::open(std::path::PathBuf::from(path)) {
                Ok(v) => match v.decode() {
                    Ok(v) => v,
                    Err(e) => return Err(format!("Unable to decode Image: {}", e))
                }
                Err(e) => {
                    return Err(format!("Unable to open image path: {}", e))
                }
            };

            img = img.resize_to_fill(dimensions.0 as u32, dimensions.1 as u32, image::imageops::FilterType::Lanczos3);
            img = img.thumbnail_exact(dimensions.0 as u32, dimensions.1 as u32);

            let width = img.width();
            let height = img.height();

            // im too proud to delete this code
            /*
            let width_multi = width / dimensions.0 as u32;
            let height_multi = height / dimensions.1 as u32;
            
            let width_respective_height = dimensions.1 as u32 * width_multi;
            let height_respective_width = dimensions.0 as u32 * height_multi;

            let cut_multi = if width_respective_height <= height {
                width_multi
            }
            else {
                height_multi
            };

            let (cut_width, cut_height) = (dimensions.0 as u32 * cut_multi, dimensions.1 as u32 * cut_multi);

            image = image.crop(
                (width - cut_width) / 2, (height - cut_height) / 2, cut_width, cut_height
            ); */

            let rgba = img.into_rgba8();
            let pixels = rgba.pixels().map(|v| {
                let c = v.channels();
                ((c[3] as u32) << 24) | ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32)
            }).collect::<Vec<u32>>();

            let image = raqote::Image {
                width: width as i32,
                height: height as i32,
                data: &pixels
            };

            target.draw_image_at(0., 0., &image, &raqote::DrawOptions::new());

            //target.fill_rect(0., 0., dimensions.0 as f32, dimensions.1 as f32, &Source::Solid(SolidSource::from_unpremultiplied_argb(255, rng.gen(), 0, 0)), &DrawOptions::new());
            Ok(target.into_vec())

}