use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Debug, Clone)]
pub struct Metadata {
    title: String,
    pub date: String,
    summary: String,
    pub link: String,
}

use pulldown_cmark::CodeBlockKind::Fenced;
use pulldown_cmark::CowStr;
use pulldown_cmark::Event;
use pulldown_cmark::Options;
use pulldown_cmark::Tag;
use pulldown_cmark::TagEnd;
use pulldown_cmark::{html, Parser};
use serde_yaml;

use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

use tokio::fs::File;
use tokio::io::AsyncReadExt;

// 实现Deserialize，以便可以自定义反序列化过程
impl<'de> Deserialize<'de> for Metadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 首先，定义一个只包含`title`的临时结构体，因为这是我们将从YAML中直接反序列化的唯一字段
        #[derive(Deserialize)]
        struct TempMetadata {
            title: String,
            date: String,
            summary: String,
        }

        // 反序列化到临时结构体
        let temp_meta = TempMetadata::deserialize(deserializer)?;

        // 然后，基于title生成link
        let link = temp_meta.title.to_lowercase().replace(" ", "-") + "-" + temp_meta.date.as_str();
        // 最后，返回填充完整的Metadata实例
        Ok(Metadata {
            title: temp_meta.title,
            date: temp_meta.date,
            summary: temp_meta.summary,
            link,
        })
    }
}

pub async fn get_post_mata_by_path(path: &str) -> Result<Metadata, Box<dyn std::error::Error>> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    let mut parts = contents.split("---");
    parts.next();
    let meta = parts.next().unwrap();
    let meta: Metadata = serde_yaml::from_str(meta)?;
    Ok(meta)
}

pub async fn get_all_post_meta() -> Result<Vec<Metadata>, Box<dyn std::error::Error>> {
    let path = "blog/";
    // 筛选出所有的md文件
    let mut files = tokio::fs::read_dir(path).await?;
    let mut metas = Vec::new();
    while let Some(file) = files.next_entry().await? {
        let path = file.path();
        if path.extension().unwrap_or_default() != "md" {
            continue;
        }
        let path = path.to_str().unwrap();
        let meta = get_post_mata_by_path(path).await?;
        metas.push(meta);
    }
    Ok(metas)
}

pub async fn covert_link_to_path(link: &str) -> Result<String, Box<dyn std::error::Error>> {
    let path = "blog/";
    let mut files = tokio::fs::read_dir(path).await?;
    while let Some(file) = files.next_entry().await? {
        let path = file.path();
        if path.extension().unwrap_or_default() != "md" {
            continue;
        }
        let path = path.to_str().unwrap();
        let meta = get_post_mata_by_path(path).await?;
        if meta.link == link {
            return Ok(path.to_string());
        }
    }
    Err("Not Found".into())
}

pub async fn get_post_by_path(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    let mut html_output = String::new();
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    let parser = Parser::new_ext(&contents, options);
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    let mut new_p = Vec::new();
    let mut to_highlight = String::new();
    let mut in_code_block = false;
    let mut current_lang = None;

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(info)) => {
                in_code_block = true;
                current_lang = match info {
                    Fenced(info) => Some(info.to_string()),
                    _ => None,
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                if in_code_block {
                    let syntax = current_lang
                        .as_ref()
                        .and_then(|lang| ss.find_syntax_by_token(lang))
                        .unwrap_or_else(|| ss.find_syntax_plain_text());
                    let highlighted =
                        highlighted_html_for_string(&to_highlight, &ss, syntax, theme);
                    new_p.push(Event::Html(CowStr::from(highlighted?)));
                    to_highlight.clear();
                    in_code_block = false;
                    current_lang = None;
                }
            }
            Event::Text(text) => {
                if in_code_block {
                    to_highlight.push_str(&text);
                } else {
                    new_p.push(Event::Text(text));
                }
            }
            _ => {
                new_p.push(event);
            }
        }
    }

    html::push_html(&mut html_output, new_p.into_iter());
    Ok(html_output)
}

#[cfg(test)]
mod tests {
    use super::*;
}
