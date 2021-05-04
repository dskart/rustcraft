use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead},
    path::Path,
};

use anyhow::*;

use crate::texture::*;

fn init_parameter(parameters: &HashMap<String, String>, key: &str) -> Result<i32> {
    if let Some(value) = parameters.get(key) {
        let value_int: i32 = value.as_str().trim().parse::<i32>()?;
        return Ok(value_int);
    }
    bail!("Error could not get parameter.");
}
#[derive(Default, Debug)]
pub struct Glyph {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub x_offset: i32,
    pub y_offset: i32,
    pub x_advance: i32,
}

impl Glyph {
    fn new(parameters: &HashMap<String, String>) -> Result<Self> {
        let glyph = Self {
            x: init_parameter(parameters, "x")?,
            y: init_parameter(parameters, "y")?,
            width: init_parameter(parameters, "width")?,
            height: init_parameter(parameters, "height")?,
            x_offset: init_parameter(parameters, "xoffset")?,
            y_offset: init_parameter(parameters, "yoffset")?,
            x_advance: init_parameter(parameters, "xadvance")?,
            id: init_parameter(parameters, "id")? as u32,
        };
        Ok(glyph)
    }
}

#[derive(Default, Debug)]
pub struct CommonParameters {
    pub line_height: i32,
    pub texture_width: i32,
    pub texture_height: i32,
}

impl CommonParameters {
    fn new(parameters: &HashMap<String, String>) -> Result<Self> {
        let common_parameters = Self {
            line_height: init_parameter(parameters, "lineHeight")?,
            texture_width: init_parameter(parameters, "scaleW")?,
            texture_height: init_parameter(parameters, "scaleH")?,
        };
        Ok(common_parameters)
    }
}

pub struct BitmapFont {
    pub common_parameters: CommonParameters,
    glyphs: HashMap<char, Glyph>,

    pub diffuse_texture: Texture,
    pub diffuse_bind_group: wgpu::BindGroup,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
}

enum LineType {
    Info(HashMap<String, String>),
    Common(HashMap<String, String>),
    Page(HashMap<String, String>),
    Chars(HashMap<String, String>),
    Char(HashMap<String, String>),
}

impl BitmapFont {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self> {
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: false,
                    },
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let diffuse_bytes = include_bytes!("../assets/fonts/vsr.png");
        let diffuse_texture = Texture::from_bytes(&device, &queue, diffuse_bytes, "vsr.png").unwrap();

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let mut glyphs: HashMap<char, Glyph> = HashMap::new();
        let mut common_parameters: Option<CommonParameters> = None;

        if let Ok(lines) = BitmapFont::read_lines("./assets/fonts/vsr.fnt") {
            for line in lines {
                if let Ok(line) = line {
                    match BitmapFont::parse_line(line) {
                        Ok(LineType::Common(ref parameters)) => {
                            common_parameters = Some(CommonParameters::new(parameters)?);
                        }
                        Ok(LineType::Char(ref parameters)) => {
                            let glyph = Glyph::new(parameters)?;
                            if let Some(c_key) = std::char::from_u32(glyph.id) {
                                glyphs.insert(c_key, glyph);
                            } else {
                                bail!(format!("Error could not get char from unicode scalar '{}'", glyph.id))
                            }
                        }
                        Ok(_) => {}
                        Err(e) => return Err(e),
                    }
                } else {
                    bail!("Error could not read line.")
                }
            }
        } else {
            bail!("Error could not read font descriptor");
        }

        if let Some(common_parameters) = common_parameters {
            if !glyphs.is_empty() {
                Ok(Self {
                    common_parameters,
                    glyphs,
                    diffuse_texture,
                    diffuse_bind_group,
                    texture_bind_group_layout,
                })
            } else {
                bail!("Error could note find any char.")
            }
        } else {
            bail!("Error could not find common parameters.")
        }
    }

    fn read_lines<P>(filename: P) -> Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    fn parse_line(line: String) -> Result<LineType> {
        enum State {
            ParsingType,
            ParsingKey,
            ParsingValue(String),
        }
        let mut state = State::ParsingType;
        let mut line_type_descriptor = "".to_string();
        let mut token = "".to_string();
        let mut escaped = false;
        let mut quoted = false;
        let mut parameters: HashMap<String, String> = HashMap::new();

        for character in line.trim().chars() {
            let c = character.to_string();
            match state {
                State::ParsingType => {
                    if c == " " {
                        line_type_descriptor = token;
                        token = "".to_string();
                        state = State::ParsingKey;
                    } else {
                        token += &c;
                    }
                }
                State::ParsingKey => {
                    if c == "=" {
                        state = State::ParsingValue(token);
                        token = "".to_string();
                    } else if c != " " {
                        token += &c;
                    }
                }
                State::ParsingValue(ref key) => {
                    if token == "" && !quoted && c == "\"" {
                        quoted = true;
                    } else if !escaped && c == "\\" {
                        escaped = true;
                    } else if !escaped && ((quoted && c == "\"") || (!quoted && c == " ")) {
                        parameters.insert(key.clone(), token);
                        token = "".to_string();
                        quoted = false;
                        state = State::ParsingKey;
                    } else {
                        token += &c;
                        escaped = false;
                    }
                }
            }
        }

        let line_type = match line_type_descriptor.as_str() {
            "info" => LineType::Info(parameters),
            "common" => LineType::Common(parameters),
            "page" => LineType::Page(parameters),
            "chars" => LineType::Chars(parameters),
            "char" => LineType::Char(parameters),
            _ => bail!(format!("Error line type '{}' is not recognized", line_type_descriptor)),
        };

        return Ok(line_type);
    }

    pub fn get_glyph(&self, c_key: &char) -> Result<&Glyph> {
        if let Some(glyph) = self.glyphs.get(c_key) {
            return Ok(glyph);
        } else {
            bail!(format!("Error could get gluph from char '{}'", c_key.to_string()))
        }
    }
}

pub struct DisplayParameters {
    pub display_text: String,
    pub x: f32,
    pub y: f32,
    pub x_scale: f32,
    pub y_scale: f32,
}

impl DisplayParameters {
    pub fn new(display_text: String, x: f32, y: f32, x_scale: f32, y_scale: f32) -> Self {
        Self {
            display_text,
            x,
            y,
            x_scale,
            y_scale,
        }
    }
}
