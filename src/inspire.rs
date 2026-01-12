//! Inspire mode - Extract design patterns from websites
//!
//! Analyzes colors, typography, spacing, and layout patterns

use anyhow::{Context, Result};
use serde::Serialize;

use crate::cdp::CdpConnection;
use crate::config::Config;
use crate::output::{
    AnimationInfo, ColorInfo, DesignInspiration, Formatter, LayoutInfo, SpacingInfo, TypographyInfo,
};

/// Run inspire mode on a URL
pub async fn run_inspire(
    cdp: &CdpConnection,
    config: &Config,
    url: &str,
    component: Option<&str>,
    save_name: Option<&str>,
    formatter: &Formatter,
) -> Result<()> {
    // Navigate to URL
    cdp.navigate(url).await?;

    // Wait for page to load
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    // Extract design data
    let inspiration = extract_design(cdp, url, component).await?;

    // Save if requested
    let screenshot_path = if let Some(name) = save_name {
        let save_dir = config.inspirations_dir().join(name);
        std::fs::create_dir_all(&save_dir)?;

        // Save screenshot
        let screenshot_data = cdp.screenshot(false).await?;
        let screenshot_path = save_dir.join("screenshot.png");
        std::fs::write(&screenshot_path, &screenshot_data)?;

        // Save design data as JSON
        let json_path = save_dir.join("design.json");
        let json = serde_json::to_string_pretty(&inspiration)?;
        std::fs::write(&json_path, json)?;

        Some(screenshot_path.to_string_lossy().to_string())
    } else {
        None
    };

    // Create final inspiration with screenshot path
    let final_inspiration = DesignInspiration {
        url: inspiration.url,
        colors: inspiration.colors,
        typography: inspiration.typography,
        spacing: inspiration.spacing,
        layout: inspiration.layout,
        animations: inspiration.animations,
        screenshot_path,
    };

    if formatter.is_json() {
        formatter.output_json(&final_inspiration);
    } else {
        println!("{}", final_inspiration);
    }

    Ok(())
}

/// Internal design data (before screenshot path)
#[derive(Debug, Serialize)]
struct DesignData {
    url: String,
    colors: Vec<ColorInfo>,
    typography: Vec<TypographyInfo>,
    spacing: SpacingInfo,
    layout: LayoutInfo,
    animations: AnimationInfo,
}

/// Extract design patterns from the page
async fn extract_design(
    cdp: &CdpConnection,
    url: &str,
    component: Option<&str>,
) -> Result<DesignData> {
    // JavaScript to extract design patterns
    let selector = component.unwrap_or("body");

    let js = format!(
        r#"
        (function() {{
            const root = document.querySelector('{}');
            if (!root) return null;

            const colors = new Map();
            const fonts = new Map();
            const paddings = new Set();
            const margins = new Set();
            const gaps = new Set();

            // Layout tracking
            let flexContainers = 0;
            let gridContainers = 0;
            const flexDirections = new Set();
            const gridTemplates = new Set();
            const justifyContent = new Set();
            const alignItems = new Set();

            // Animation tracking
            const timingFunctions = new Set();
            const durations = new Set();
            const transitions = new Set();

            function walkElements(el) {{
                const style = getComputedStyle(el);

                // Colors
                ['color', 'backgroundColor', 'borderColor'].forEach(prop => {{
                    const val = style[prop];
                    if (val && val !== 'rgba(0, 0, 0, 0)' && val !== 'transparent') {{
                        const hex = rgbToHex(val);
                        if (hex) {{
                            const key = hex + '|' + prop;
                            colors.set(key, (colors.get(key) || 0) + 1);
                        }}
                    }}
                }});

                // Typography
                const fontKey = [
                    style.fontFamily.split(',')[0].trim().replace(/['"]/g, ''),
                    style.fontSize,
                    style.fontWeight,
                    style.lineHeight
                ].join('|');
                fonts.set(fontKey, (fonts.get(fontKey) || 0) + 1);

                // Spacing
                ['paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft'].forEach(p => {{
                    const val = style[p];
                    if (val && val !== '0px') paddings.add(val);
                }});
                ['marginTop', 'marginRight', 'marginBottom', 'marginLeft'].forEach(m => {{
                    const val = style[m];
                    if (val && val !== '0px' && !val.startsWith('-')) margins.add(val);
                }});
                if (style.gap && style.gap !== 'normal') gaps.add(style.gap);

                // Layout - Flex
                if (style.display === 'flex' || style.display === 'inline-flex') {{
                    flexContainers++;
                    if (style.flexDirection && style.flexDirection !== 'row') {{
                        flexDirections.add(style.flexDirection);
                    }}
                    if (style.justifyContent && style.justifyContent !== 'normal' && style.justifyContent !== 'flex-start') {{
                        justifyContent.add(style.justifyContent);
                    }}
                    if (style.alignItems && style.alignItems !== 'normal' && style.alignItems !== 'stretch') {{
                        alignItems.add(style.alignItems);
                    }}
                }}

                // Layout - Grid
                if (style.display === 'grid' || style.display === 'inline-grid') {{
                    gridContainers++;
                    if (style.gridTemplateColumns && style.gridTemplateColumns !== 'none') {{
                        // Simplify complex templates
                        const simplified = style.gridTemplateColumns.length > 50
                            ? style.gridTemplateColumns.substring(0, 50) + '...'
                            : style.gridTemplateColumns;
                        gridTemplates.add(simplified);
                    }}
                }}

                // Animations
                if (style.transitionTimingFunction && style.transitionTimingFunction !== 'ease') {{
                    timingFunctions.add(style.transitionTimingFunction);
                }}
                if (style.transitionDuration && style.transitionDuration !== '0s') {{
                    durations.add(style.transitionDuration);
                }}
                if (style.transition && style.transition !== 'none' && style.transition !== 'all 0s ease 0s') {{
                    // Simplify transition strings
                    const parts = style.transition.split(',').slice(0, 3);
                    parts.forEach(t => {{
                        const simplified = t.trim().length > 40 ? t.trim().substring(0, 40) + '...' : t.trim();
                        if (simplified && simplified !== 'all 0s ease 0s') transitions.add(simplified);
                    }});
                }}
                if (style.animationTimingFunction && style.animationTimingFunction !== 'ease') {{
                    timingFunctions.add(style.animationTimingFunction);
                }}
                if (style.animationDuration && style.animationDuration !== '0s') {{
                    durations.add(style.animationDuration);
                }}

                // Recurse into children (limit depth)
                if (el.children.length < 100) {{
                    for (const child of el.children) {{
                        walkElements(child);
                    }}
                }}
            }}

            function rgbToHex(rgb) {{
                const match = rgb.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
                if (!match) return null;
                const hex = '#' + [match[1], match[2], match[3]]
                    .map(x => parseInt(x).toString(16).padStart(2, '0'))
                    .join('');
                return hex;
            }}

            walkElements(root);

            // Process colors
            const colorList = [];
            colors.forEach((count, key) => {{
                const [hex, usage] = key.split('|');
                colorList.push({{ hex, usage, count }});
            }});
            colorList.sort((a, b) => b.count - a.count);

            // Process fonts
            const fontList = [];
            fonts.forEach((count, key) => {{
                const [family, size, weight, lineHeight] = key.split('|');
                if (count >= 2) {{
                    fontList.push({{
                        font_family: family,
                        font_size: size,
                        font_weight: weight,
                        line_height: lineHeight,
                        usage: count > 10 ? 'primary' : count > 5 ? 'secondary' : 'accent'
                    }});
                }}
            }});
            fontList.sort((a, b) => {{
                const aCount = fonts.get([a.font_family, a.font_size, a.font_weight, a.line_height].join('|'));
                const bCount = fonts.get([b.font_family, b.font_size, b.font_weight, b.line_height].join('|'));
                return bCount - aCount;
            }});

            return {{
                colors: colorList.slice(0, 15),
                typography: fontList.slice(0, 10),
                spacing: {{
                    padding_values: [...paddings].sort((a,b) => parseInt(a) - parseInt(b)).slice(0, 10),
                    margin_values: [...margins].sort((a,b) => parseInt(a) - parseInt(b)).slice(0, 10),
                    gap_values: [...gaps].sort((a,b) => parseInt(a) - parseInt(b)).slice(0, 5)
                }},
                layout: {{
                    flex_containers: flexContainers,
                    grid_containers: gridContainers,
                    flex_directions: [...flexDirections].slice(0, 5),
                    grid_templates: [...gridTemplates].slice(0, 5),
                    justify_content: [...justifyContent].slice(0, 5),
                    align_items: [...alignItems].slice(0, 5)
                }},
                animations: {{
                    timing_functions: [...timingFunctions].slice(0, 5),
                    durations: [...durations].slice(0, 5),
                    transitions: [...transitions].slice(0, 5)
                }}
            }};
        }})()
    "#,
        selector
    );

    let result = cdp
        .evaluate(&js)
        .await
        .context("Failed to extract design patterns")?;

    if result.is_null() {
        return Err(anyhow::anyhow!(
            "No element matches selector \"{}\"",
            selector
        ));
    }

    // Parse the result
    let colors: Vec<ColorInfo> = result
        .get("colors")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let typography: Vec<TypographyInfo> = result
        .get("typography")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let spacing_raw = result
        .get("spacing")
        .cloned()
        .unwrap_or(serde_json::json!({}));
    let spacing = SpacingInfo {
        padding_values: spacing_raw
            .get("padding_values")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
        margin_values: spacing_raw
            .get("margin_values")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
        gap_values: spacing_raw
            .get("gap_values")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
    };

    let layout_raw = result
        .get("layout")
        .cloned()
        .unwrap_or(serde_json::json!({}));
    let layout = LayoutInfo {
        flex_containers: layout_raw
            .get("flex_containers")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        grid_containers: layout_raw
            .get("grid_containers")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32,
        flex_directions: layout_raw
            .get("flex_directions")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
        grid_templates: layout_raw
            .get("grid_templates")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
        justify_content: layout_raw
            .get("justify_content")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
        align_items: layout_raw
            .get("align_items")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
    };

    let animations_raw = result
        .get("animations")
        .cloned()
        .unwrap_or(serde_json::json!({}));
    let animations = AnimationInfo {
        timing_functions: animations_raw
            .get("timing_functions")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
        durations: animations_raw
            .get("durations")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
        transitions: animations_raw
            .get("transitions")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
    };

    Ok(DesignData {
        url: url.to_string(),
        colors,
        typography,
        spacing,
        layout,
        animations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_design_data_serialize() {
        let data = DesignData {
            url: "https://example.com".to_string(),
            colors: vec![],
            typography: vec![],
            spacing: SpacingInfo::default(),
            layout: LayoutInfo::default(),
            animations: AnimationInfo::default(),
        };

        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("example.com"));
        assert!(json.contains("colors"));
        assert!(json.contains("typography"));
    }

    #[test]
    fn test_design_data_with_content() {
        let data = DesignData {
            url: "https://test.com".to_string(),
            colors: vec![ColorInfo {
                hex: "#ff0000".to_string(),
                usage: "background".to_string(),
                count: 5,
            }],
            typography: vec![TypographyInfo {
                font_family: "Arial".to_string(),
                font_size: "16px".to_string(),
                font_weight: "400".to_string(),
                line_height: "1.5".to_string(),
                usage: "body".to_string(),
            }],
            spacing: SpacingInfo {
                padding_values: vec!["8px".to_string()],
                margin_values: vec!["16px".to_string()],
                gap_values: vec!["12px".to_string()],
            },
            layout: LayoutInfo {
                flex_containers: 1,
                grid_containers: 0,
                flex_directions: vec!["row".to_string()],
                grid_templates: vec![],
                justify_content: vec!["center".to_string()],
                align_items: vec!["center".to_string()],
            },
            animations: AnimationInfo {
                timing_functions: vec!["ease".to_string()],
                durations: vec!["0.3s".to_string()],
                transitions: vec!["all 0.3s ease".to_string()],
            },
        };

        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("#ff0000"));
        assert!(json.contains("Arial"));
        assert!(json.contains("row"));
        assert!(json.contains("ease"));
    }
}
