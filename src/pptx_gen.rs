//! GitHub README → PowerPoint 自動生成（純 Rust + zip クレート）
//!
//! Node.js / Python など外部ランタイムは一切使用しない。
//! PPTX (OOXML) は ZIP + XML のため、Rust の `zip` クレートで直接バイト列を生成する。
//!
//! ルール: Node.js 使用禁止・Rust + Poem のみ

use std::io::{Cursor, Write};

use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::github_projects::GithubProject;
use crate::state::AppState;

// ─────────────────────────────────────────────────────────
// 公開型
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideContent {
    pub title: String,
    pub bullets: Vec<String>,
    pub notes: Option<String>,
    pub layout: SlideLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SlideLayout {
    Cover,       // 表紙（暗い背景）
    Content,     // タイトル + 箇条書き
    Stats,       // 技術スタックカード
    TwoCol,      // 2カラム
    Closing,     // 締め
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PptxSpec {
    pub title: String,
    pub subtitle: String,
    pub theme: PptxTheme,
    pub slides: Vec<SlideContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PptxTheme {
    /// 背景色（RRGGBB hex）
    pub bg: String,
    /// タイトル文字色
    pub title_color: String,
    /// 本文文字色
    pub body_color: String,
    /// アクセント色
    pub accent: String,
}

impl Default for PptxTheme {
    fn default() -> Self {
        // Midnight Executive
        Self {
            bg: "1E2761".into(),
            title_color: "FFFFFF".into(),
            body_color: "CADCFC".into(),
            accent: "00B4D8".into(),
        }
    }
}

// ─────────────────────────────────────────────────────────
// README → PptxSpec 変換
// ─────────────────────────────────────────────────────────

pub async fn build_spec_from_projects(
    projects: &[GithubProject],
    lang: &str,
) -> anyhow::Result<PptxSpec> {
    if projects.is_empty() {
        anyhow::bail!("プロジェクトが選択されていません");
    }
    let is_ja = lang == "ja";
    Ok(if projects.len() == 1 {
        single_project_spec(&projects[0], is_ja)
    } else {
        multi_project_spec(projects, is_ja)
    })
}

fn single_project_spec(p: &GithubProject, is_ja: bool) -> PptxSpec {
    let readme = p.readme_content.as_deref().unwrap_or("");
    let sections = parse_readme_sections(readme);
    let mut slides = Vec::new();

    // 表紙
    slides.push(SlideContent {
        title: p.name.clone(),
        bullets: vec![
            p.description.clone().unwrap_or_default(),
            format!("⭐ {} stars  |  {}", p.stargazers_count, p.language.as_deref().unwrap_or("N/A")),
            format!("github.com/{}", p.full_name),
        ],
        notes: None,
        layout: SlideLayout::Cover,
    });

    // README セクションをスライドに変換（最大 7 枚）
    for (heading, body) in sections.iter().take(7) {
        if heading.to_lowercase().contains("license") { continue; }
        let bullets = body_to_bullets(body, 6);
        if bullets.is_empty() { continue; }
        slides.push(SlideContent {
            title: heading.clone(),
            bullets,
            notes: None,
            layout: SlideLayout::Content,
        });
    }

    // 技術スタックスライド
    if let Some(ext_json) = &p.ext_summary {
        if let Ok(exts) = serde_json::from_str::<Vec<(String, usize)>>(ext_json) {
            let bullets: Vec<String> = exts.iter().take(8)
                .map(|(e, c)| format!(".{e}  —  {c} files"))
                .collect();
            if !bullets.is_empty() {
                slides.push(SlideContent {
                    title: if is_ja { "技術スタック".into() } else { "Tech Stack".into() },
                    bullets,
                    notes: None,
                    layout: SlideLayout::Stats,
                });
            }
        }
    }

    // 締め
    slides.push(SlideContent {
        title: if is_ja { "ありがとうございました".into() } else { "Thank You".into() },
        bullets: vec![
            format!("🔗 github.com/{}", p.full_name),
            format!("📅 {}", &p.updated_at_gh[..10]),
        ],
        notes: None,
        layout: SlideLayout::Closing,
    });

    PptxSpec {
        title: p.name.clone(),
        subtitle: p.description.clone().unwrap_or_default(),
        theme: theme_by_language(p.language.as_deref()),
        slides,
    }
}

fn multi_project_spec(projects: &[GithubProject], is_ja: bool) -> PptxSpec {
    let mut slides = Vec::new();

    slides.push(SlideContent {
        title: if is_ja { "GitHub プロジェクト一覧".into() } else { "GitHub Projects".into() },
        bullets: vec![format!("{} repositories", projects.len())],
        notes: None,
        layout: SlideLayout::Cover,
    });

    for p in projects.iter().take(12) {
        let first = p.readme_content.as_deref()
            .map(|r| {
                let secs = parse_readme_sections(r);
                secs.into_iter().next()
                    .map(|(_, b)| body_to_bullets(&b, 4))
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        let mut bullets = vec![
            p.description.clone().unwrap_or_default(),
            format!("⭐ {}  |  {}  |  {}", p.stargazers_count, p.language.as_deref().unwrap_or("N/A"), &p.updated_at_gh[..10]),
        ];
        bullets.extend(first);

        slides.push(SlideContent {
            title: p.name.clone(),
            bullets: bullets.into_iter().filter(|b| !b.is_empty()).collect(),
            notes: Some(p.html_url.clone()),
            layout: SlideLayout::Content,
        });
    }

    slides.push(SlideContent {
        title: if is_ja { "まとめ" } else { "Summary" }.into(),
        bullets: projects.iter().map(|p| format!("• {}", p.name)).collect(),
        notes: None,
        layout: SlideLayout::Closing,
    });

    PptxSpec {
        title: if is_ja { "GitHub プロジェクト一覧".into() } else { "GitHub Projects".into() },
        subtitle: if is_ja { "aruaru 自動生成".into() } else { "Generated by aruaru".into() },
        theme: PptxTheme::default(),
        slides,
    }
}

fn parse_readme_sections(readme: &str) -> Vec<(String, String)> {
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut heading = String::new();
    let mut body_lines: Vec<&str> = Vec::new();
    let mut in_code = false;

    for line in readme.lines() {
        if line.starts_with("```") || line.starts_with("~~~") {
            in_code = !in_code;
            continue;
        }
        if in_code { continue; }

        if let Some(rest) = line.strip_prefix("## ").or_else(|| line.strip_prefix("### ")) {
            if !heading.is_empty() {
                sections.push((heading.clone(), body_lines.join("\n").trim().to_string()));
                body_lines.clear();
            }
            heading = rest.trim().to_string();
        } else if !heading.is_empty() {
            body_lines.push(line);
        } else if !line.starts_with('#') && !line.is_empty() && heading.is_empty() {
            heading = "Overview".into();
            body_lines.push(line);
        }
    }
    if !heading.is_empty() {
        sections.push((heading, body_lines.join("\n").trim().to_string()));
    }
    sections
}

fn body_to_bullets(body: &str, max: usize) -> Vec<String> {
    body.lines()
        .map(|l| l.trim_start_matches(['-', '*', ' ', '\t']).trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('[') && l.len() > 4)
        .take(max)
        .collect()
}

fn theme_by_language(lang: Option<&str>) -> PptxTheme {
    match lang.unwrap_or("").to_lowercase().as_str() {
        "rust" => PptxTheme { bg: "B85042".into(), title_color: "FFFFFF".into(), body_color: "F0E0DD".into(), accent: "FFD166".into() },
        "typescript" | "javascript" => PptxTheme { bg: "065A82".into(), title_color: "FFFFFF".into(), body_color: "CADCFC".into(), accent: "00B4D8".into() },
        "python" => PptxTheme { bg: "2C5F2D".into(), title_color: "FFFFFF".into(), body_color: "D8F3DC".into(), accent: "95D5B2".into() },
        "go" => PptxTheme { bg: "028090".into(), title_color: "FFFFFF".into(), body_color: "E0FBFC".into(), accent: "02C39A".into() },
        "java" | "kotlin" => PptxTheme { bg: "990011".into(), title_color: "FFFFFF".into(), body_color: "FCF6F5".into(), accent: "FF6B6B".into() },
        _ => PptxTheme::default(),
    }
}

// ─────────────────────────────────────────────────────────
// 純 Rust PPTX 生成（zip + OOXML）
// ─────────────────────────────────────────────────────────

/// PptxSpec を PPTX バイト列に変換する（Node.js 不使用・純 Rust）
pub fn render_pptx(spec: &PptxSpec) -> anyhow::Result<Vec<u8>> {
    let buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let n = spec.slides.len();

    // ── [Content_Types].xml ───────────────────────────────
    zip.start_file("[Content_Types].xml", opts)?;
    let mut ct = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml"  ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml"
    ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/ppt/theme/theme1.xml"
    ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
  <Override PartName="/ppt/slideMasters/slideMaster1.xml"
    ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
  <Override PartName="/ppt/slideLayouts/slideLayout1.xml"
    ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
"#);
    for i in 1..=n {
        ct.push_str(&format!(
            r#"  <Override PartName="/ppt/slides/slide{i}.xml"
    ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
"#
        ));
    }
    ct.push_str("</Types>");
    zip.write_all(ct.as_bytes())?;

    // ── _rels/.rels ───────────────────────────────────────
    zip.start_file("_rels/.rels", opts)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument"
    Target="ppt/presentation.xml"/>
</Relationships>"#,
    )?;

    // ── ppt/theme/theme1.xml ──────────────────────────────
    zip.start_file("ppt/theme/theme1.xml", opts)?;
    zip.write_all(theme_xml(&spec.theme).as_bytes())?;

    // ── ppt/slideMasters/slideMaster1.xml ─────────────────
    zip.start_file("ppt/slideMasters/slideMaster1.xml", opts)?;
    zip.write_all(slide_master_xml(&spec.theme).as_bytes())?;

    zip.start_file("ppt/slideMasters/_rels/slideMaster1.xml.rels", opts)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout"
    Target="../slideLayouts/slideLayout1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme"
    Target="../theme/theme1.xml"/>
</Relationships>"#,
    )?;

    // ── ppt/slideLayouts/slideLayout1.xml ─────────────────
    zip.start_file("ppt/slideLayouts/slideLayout1.xml", opts)?;
    zip.write_all(slide_layout_xml().as_bytes())?;

    zip.start_file("ppt/slideLayouts/_rels/slideLayout1.xml.rels", opts)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster"
    Target="../slideMasters/slideMaster1.xml"/>
</Relationships>"#,
    )?;

    // ── ppt/presentation.xml ──────────────────────────────
    zip.start_file("ppt/presentation.xml", opts)?;
    zip.write_all(presentation_xml(n).as_bytes())?;

    zip.start_file("ppt/_rels/presentation.xml.rels", opts)?;
    zip.write_all(presentation_rels_xml(n).as_bytes())?;

    // ── スライド ──────────────────────────────────────────
    for (i, slide) in spec.slides.iter().enumerate() {
        let num = i + 1;
        zip.start_file(format!("ppt/slides/slide{num}.xml"), opts)?;
        zip.write_all(slide_xml(slide, &spec.theme).as_bytes())?;

        zip.start_file(format!("ppt/slides/_rels/slide{num}.xml.rels"), opts)?;
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout"
    Target="../slideLayouts/slideLayout1.xml"/>
</Relationships>"#,
        )?;
    }

    let cursor = zip.finish()?;
    Ok(cursor.into_inner())
}

// ─────────────────────────────────────────────────────────
// OOXML テンプレート関数
// ─────────────────────────────────────────────────────────

fn theme_xml(t: &PptxTheme) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="aruaru">
  <a:themeElements>
    <a:clrScheme name="aruaru">
      <a:dk1><a:sysClr lastClr="000000" val="windowText"/></a:dk1>
      <a:lt1><a:sysClr lastClr="FFFFFF" val="window"/></a:lt1>
      <a:dk2><a:srgbClr val="{bg}"/></a:dk2>
      <a:lt2><a:srgbClr val="FFFFFF"/></a:lt2>
      <a:accent1><a:srgbClr val="{accent}"/></a:accent1>
      <a:accent2><a:srgbClr val="{title}"/></a:accent2>
      <a:accent3><a:srgbClr val="{body}"/></a:accent3>
      <a:accent4><a:srgbClr val="{accent}"/></a:accent4>
      <a:accent5><a:srgbClr val="{accent}"/></a:accent5>
      <a:accent6><a:srgbClr val="{accent}"/></a:accent6>
      <a:hlink><a:srgbClr val="{accent}"/></a:hlink>
      <a:folHlink><a:srgbClr val="{accent}"/></a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="aruaru">
      <a:majorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont>
      <a:minorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont>
    </a:fontScheme>
    <a:fmtScheme name="Office">
      <a:fillStyleLst>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
      </a:fillStyleLst>
      <a:lnStyleLst>
        <a:ln w="6350"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>
        <a:ln w="12700"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>
        <a:ln w="19050"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>
      </a:lnStyleLst>
      <a:effectStyleLst>
        <a:effectStyle><a:effectLst/></a:effectStyle>
        <a:effectStyle><a:effectLst/></a:effectStyle>
        <a:effectStyle><a:effectLst/></a:effectStyle>
      </a:effectStyleLst>
      <a:bgFillStyleLst>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
      </a:bgFillStyleLst>
    </a:fmtScheme>
  </a:themeElements>
</a:theme>"#,
        bg = t.bg, accent = t.accent, title = t.title_color, body = t.body_color,
    )
}

fn slide_master_xml(t: &PptxTheme) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
             xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld><p:bg><p:bgPr>
    <a:solidFill><a:srgbClr val="{bg}"/></a:solidFill>
  </p:bgPr></p:bg><p:spTree>
    <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
    <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
  </p:spTree></p:cSld>
  <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2"
    accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
  <p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst>
  <p:txStyles>
    <p:titleStyle><a:lstStyle><a:defPPr><a:defRPr lang="ja-JP" b="1" sz="3200"><a:solidFill><a:srgbClr val="{title}"/></a:solidFill><a:latin typeface="Calibri"/></a:defRPr></a:defPPr></a:lstStyle></p:titleStyle>
    <p:bodyStyle><a:lstStyle><a:defPPr><a:defRPr lang="ja-JP" sz="1800"><a:solidFill><a:srgbClr val="{body}"/></a:solidFill><a:latin typeface="Calibri"/></a:defRPr></a:defPPr></a:lstStyle></p:bodyStyle>
    <p:otherStyle><a:lstStyle/></p:otherStyle>
  </p:txStyles>
</p:sldMaster>"#,
        bg = t.bg, title = t.title_color, body = t.body_color,
    )
}

fn slide_layout_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
             xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
             type="blank" preserve="1">
  <p:cSld name="Blank"><p:spTree>
    <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
    <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
  </p:spTree></p:cSld>
  <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sldLayout>"#.into()
}

fn presentation_xml(n_slides: usize) -> String {
    let ids: String = (1..=n_slides)
        .map(|i| format!(r#"    <p:sldId id="{}" r:id="rId{}"/>"#, 255 + i, i))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
                xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
                xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rIdM1"/></p:sldMasterIdLst>
  <p:sldSz cx="9144000" cy="5143500" type="screen16x9"/>
  <p:notesSz cx="6858000" cy="9144000"/>
  <p:sldIdLst>
{ids}
  </p:sldIdLst>
</p:presentation>"#
    )
}

fn presentation_rels_xml(n_slides: usize) -> String {
    let mut rels = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdM1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster"
    Target="slideMasters/slideMaster1.xml"/>
"#,
    );
    for i in 1..=n_slides {
        rels.push_str(&format!(
            r#"  <Relationship Id="rId{i}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide"
    Target="slides/slide{i}.xml"/>
"#
        ));
    }
    rels.push_str("</Relationships>");
    rels
}

/// スライド1枚分の XML を生成
fn slide_xml(slide: &SlideContent, theme: &PptxTheme) -> String {
    // EMU 単位（1 inch = 914400 EMU）
    // スライドサイズ: 9144000 × 5143500 EMU (16:9 ワイド)
    let (bg_color, title_color, body_color) = match slide.layout {
        SlideLayout::Cover | SlideLayout::Closing => {
            (&theme.bg, &theme.title_color, &theme.body_color)
        }
        _ => {
            // コンテンツスライドは白背景
            static WHITE: &str = "FFFFFF";
            static DARK: &str = "1A1A2E";
            static GRAY: &str = "444444";
            (WHITE, &theme.bg as &str, GRAY)
            // Rust borrow trick workaround below
        }
    };

    // 背景色
    let bg_xml = match slide.layout {
        SlideLayout::Cover | SlideLayout::Closing => format!(
            r#"<p:bg><p:bgPr><a:solidFill><a:srgbClr val="{bg}"/></a:solidFill></p:bgPr></p:bg>"#,
            bg = bg_color
        ),
        _ => format!(
            r#"<p:bg><p:bgPr><a:solidFill><a:srgbClr val="FFFFFF"/></a:solidFill></p:bgPr></p:bg>"#
        ),
    };

    static BODY_GRAY: &str = "444444";
    let (tc, bc): (&str, &str) = match slide.layout {
        SlideLayout::Cover | SlideLayout::Closing => (&theme.title_color, &theme.body_color),
        _ => (&theme.bg, BODY_GRAY),
    };

    // タイトル
    let title_emu_y = match slide.layout {
        SlideLayout::Cover | SlideLayout::Closing => 1_600_000u64,
        _ => 300_000,
    };
    let title_sz = match slide.layout {
        SlideLayout::Cover => 4000u32,
        SlideLayout::Closing => 3600,
        _ => 2800,
    };

    let title_sp = text_box_xml(
        2, "Title",
        457200, title_emu_y, 8229600, 900000,
        &xml_escape(&slide.title), title_sz, tc, true,
    );

    // 本文（箇条書き）
    let body_sp = if slide.layout == SlideLayout::Cover || slide.layout == SlideLayout::Closing {
        // 中央下部に箇条書き
        let paras: String = slide.bullets.iter().map(|b| para_xml(&xml_escape(b), 1800, bc, false)).collect();
        text_box_with_paras_xml(
            3, "Body",
            457200, 2_700_000, 8229600, 2_200_000,
            &paras,
        )
    } else {
        // 通常スライドは左寄せ箇条書き
        let paras: String = slide.bullets.iter().map(|b| bullet_para_xml(&xml_escape(b), 1600, bc)).collect();
        text_box_with_paras_xml(
            3, "Body",
            457200, 1_350_000, 8229600, 3_650_000,
            &paras,
        )
    };

    // アクセントライン（Cover / Closing 以外）
    let accent_line = match slide.layout {
        SlideLayout::Cover | SlideLayout::Closing => String::new(),
        _ => format!(
            r#"<p:sp>
  <p:nvSpPr><p:cNvPr id="4" name="Line"/><p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr><p:nvPr/></p:nvSpPr>
  <p:spPr><a:xfrm><a:off x="457200" y="1280000"/><a:ext cx="8229600" cy="30000"/></a:xfrm>
    <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
    <a:solidFill><a:srgbClr val="{accent}"/></a:solidFill>
  </p:spPr>
</p:sp>"#,
            accent = theme.accent,
        ),
    };

    // ノート
    let notes_xml = match &slide.notes {
        Some(n) if !n.is_empty() => format!(
            r#"<p:notes xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
           xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld><p:spTree>
    <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
    <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
    <p:sp>
      <p:nvSpPr><p:cNvPr id="2" name="Note"/><p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr>
        <p:nvPr><p:ph type="body" idx="1"/></p:nvPr>
      </p:nvSpPr>
      <p:spPr/>
      <p:txBody><a:bodyPr/><a:lstStyle/>
        <a:p><a:r><a:t>{}</a:t></a:r></a:p>
      </p:txBody>
    </p:sp>
  </p:spTree></p:cSld>
</p:notes>"#,
            xml_escape(n)
        ),
        _ => String::new(),
    };

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    {bg_xml}
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
      {title_sp}
      {accent_line}
      {body_sp}
    </p:spTree>
  </p:cSld>
  {notes_xml}
  <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sld>"#
    )
}

// ─── XML ヘルパー ─────────────────────────────────────────

fn text_box_xml(
    id: u32, name: &str,
    x: u64, y: u64, cx: u64, cy: u64,
    text: &str, sz: u32, color: &str, bold: bool,
) -> String {
    let b = if bold { r#" b="1""# } else { "" };
    format!(
        r#"<p:sp>
  <p:nvSpPr><p:cNvPr id="{id}" name="{name}"/><p:cNvSpPr txBox="1"><a:spLocks noGrp="1"/></p:cNvSpPr><p:nvPr/></p:nvSpPr>
  <p:spPr><a:xfrm><a:off x="{x}" y="{y}"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm>
    <a:prstGeom prst="rect"><a:avLst/></a:prstGeom><a:noFill/>
  </p:spPr>
  <p:txBody>
    <a:bodyPr wrap="square" rtlCol="0"/>
    <a:lstStyle/>
    <a:p><a:pPr algn="ctr"/><a:r>
      <a:rPr lang="ja-JP"{b} sz="{sz}"><a:solidFill><a:srgbClr val="{color}"/></a:solidFill><a:latin typeface="Calibri"/></a:rPr>
      <a:t>{text}</a:t>
    </a:r></a:p>
  </p:txBody>
</p:sp>"#
    )
}

fn text_box_with_paras_xml(
    id: u32, name: &str,
    x: u64, y: u64, cx: u64, cy: u64,
    paras: &str,
) -> String {
    format!(
        r#"<p:sp>
  <p:nvSpPr><p:cNvPr id="{id}" name="{name}"/><p:cNvSpPr txBox="1"><a:spLocks noGrp="1"/></p:cNvSpPr><p:nvPr/></p:nvSpPr>
  <p:spPr><a:xfrm><a:off x="{x}" y="{y}"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm>
    <a:prstGeom prst="rect"><a:avLst/></a:prstGeom><a:noFill/>
  </p:spPr>
  <p:txBody>
    <a:bodyPr wrap="square" rtlCol="0"/>
    <a:lstStyle/>
    {paras}
  </p:txBody>
</p:sp>"#
    )
}

fn para_xml(text: &str, sz: u32, color: &str, bold: bool) -> String {
    let b = if bold { r#" b="1""# } else { "" };
    format!(
        r#"<a:p><a:pPr algn="ctr"/><a:r>
  <a:rPr lang="ja-JP"{b} sz="{sz}"><a:solidFill><a:srgbClr val="{color}"/></a:solidFill><a:latin typeface="Calibri"/></a:rPr>
  <a:t>{text}</a:t>
</a:r></a:p>"#
    )
}

fn bullet_para_xml(text: &str, sz: u32, color: &str) -> String {
    format!(
        r#"<a:p>
  <a:pPr marL="342900" indent="-342900">
    <a:buChar char="•"/>
  </a:pPr>
  <a:r>
    <a:rPr lang="ja-JP" sz="{sz}"><a:solidFill><a:srgbClr val="{color}"/></a:solidFill><a:latin typeface="Calibri"/></a:rPr>
    <a:t>{text}</a:t>
  </a:r>
</a:p>"#
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

// ─────────────────────────────────────────────────────────
// DB ヘルパー（AppState）
// ─────────────────────────────────────────────────────────

impl AppState {
    pub async fn create_pptx_job(&self, gh_ids: &[i64]) -> anyhow::Result<i64> {
        let now  = chrono::Utc::now().to_rfc3339();
        let json = serde_json::to_string(gh_ids)?;
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO pptx_jobs (gh_ids, status, created_at) VALUES ($1,'pending',$2) RETURNING id",
        )
        .bind(&json)
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn update_pptx_job(
        &self, job_id: i64, status: &str,
        filename: Option<&str>, error_msg: Option<&str>,
    ) -> anyhow::Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE pptx_jobs SET status=$1, filename=$2, error_msg=$3, finished_at=$4 WHERE id=$5",
        )
        .bind(status).bind(filename).bind(error_msg).bind(&now).bind(job_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
