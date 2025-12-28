use super::ast::*;
use crate::page::math::text::TextType;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use finl_unicode::categories::CharacterCategories;
use itertools::Itertools;
use log::warn;

pub(super) fn render_equation(equation: Equation) -> Result<String> {
    Ok(format!(
        "<math xmlns=\"http://www.w3.org/1998/Math/MathML\">{}</math>",
        render_eq(equation)?
    ))
}

fn render_eq(eq: Equation) -> Result<String> {
    let mut content = String::new();

    for op in eq {
        content.push_str(render_op(op)?.as_ref());
    }

    Ok(content)
}

fn render_group(eq: Equation) -> Result<String> {
    match eq.len() {
        0 => Ok("<mi></mi>".to_string()),
        1 => match &eq[0] {
            MathOp::Text(text) => Ok(format!("<mrow>{}</mrow>", render_text(text.clone())?)),
            _ => eq.into_iter().map(render_op).collect::<Result<_>>(),
        },
        _ => Ok(format!("<mrow>{}</mrow>", render_eq(eq)?)),
    }
}

fn render_op(op: MathOp) -> Result<String> {
    match op {
        MathOp::Text(text) => render_text(text),
        MathOp::Accent { char, body } => render_accent(char, body),
        MathOp::Box { body, display } => render_box(body, display),
        MathOp::BoxedFormula { align, body } => render_boxed_formula(body, align),
        MathOp::Brackets {
            align,
            open,
            close,
            body,
        } => render_brackets(open, close, body, align),
        MathOp::BracketsWithSeps {
            open,
            close,
            sep,
            segments,
            align,
        } => render_brackets_with_seps(open, close, sep, segments, align),
        MathOp::EquationArray {
            align,
            columns,
            rows,
        } => render_equation_array(columns, rows, align),
        MathOp::Fraction { num, den, small } => render_fraction(num, den, small),
        MathOp::FunctionApply { func, body } => render_function_apply(func, body),
        MathOp::LeftSubSup { sub, sup, body } => render_left_sub_sup(sub, sup, body),
        MathOp::LowerLimit { body, limit } => render_lower_limit(body, limit),
        MathOp::Matrix {
            align,
            columns,
            brackets,
            items,
        } => render_matrix(columns, items, brackets, align),
        MathOp::NAry {
            op,
            sub,
            sup,
            body,
            display,
        } => render_nary(op, sub, sup, body, display),
        MathOp::OverBar { body } => render_over_bar(body),
        MathOp::Phantom {
            kind,
            display,
            body,
        } => render_phantom(kind, display, body),
        MathOp::Radical { degree, body } => render_radical(degree, body),
        MathOp::SlashedFraction { num, den, linear } => render_slashed_fraction(num, den, linear),
        MathOp::Stack { num, den } => render_stack(num, den),
        MathOp::StretchStack { char, pos, body } => render_stretch_stack(char, pos, body),
        MathOp::Subscript { sub, body } => render_subscript(sub, body),
        MathOp::SubSup {
            align,
            sub,
            sup,
            body,
        } => render_sub_sup(sub, sup, body, align),
        MathOp::Superscript { sup, body } => render_superscript(sup, body),
        MathOp::UnderBar { body } => render_under_bar(body),
        MathOp::UpperLimit { body, limit } => render_upper_limit(body, limit),
    }
}

fn render_text(text: String) -> Result<String> {
    // <mi>: identifier
    // <mo>: operator
    // <mn>: numeric

    let filtered: String = text.chars().filter(|c| !is_skippable_format(*c)).collect();
    if filtered.is_empty() {
        return Ok(String::new());
    }

    let mut parts = vec![];
    let mut current = String::new();
    let mut current_type = None;

    for (i, c) in filtered.char_indices() {
        let text_type = TextType::from_char(&c)?;
        if Some(text_type) != current_type {
            if i != 0 {
                parts.push((current_type, current));
                current = String::new();
            }

            current_type = Some(text_type);
        }

        current.push(c);
    }

    parts.push((current_type, current));

    let mut rendered_parts = Vec::with_capacity(parts.len());
    for (text_type, text) in parts {
        let rendered = match text_type {
            None => return Err(eyre!("No text type for text `{}`", text)),
            Some(TextType::Bold) => format!("<mi mathvariant=\"bold\">{}</mi>", text),
            Some(TextType::BoldItalic) => format!("<mi mathvariant=\"bold-italic\">{}</mi>", text),
            Some(TextType::BoldScript) => {
                format!("<mi mathvariant=\"bold-script\">{}</mi>", text)
            }
            Some(TextType::Double) => format!("<mi mathvariant=\"double-struck\">{}</mi>", text),
            // Some(TextType::DoubleItalic) => {
            //     warn!(
            //         "Double-struck italic not supported; falling back to double-struck with italic style: {}",
            //         text
            //     );
            //     format!(
            //         "<mi mathvariant=\"double-struck\" style=\"font-style: italic\">{}</mi>",
            //         text
            //     )
            // }
            Some(TextType::DoubleOperator) => {
                let text = match text.as_str() {
                    "\u{2145}" => "\u{1d437}",
                    "\u{2146}" => "\u{1d451}",
                    _ => {
                        warn!(
                            "Math feature not implemented: double-operator mapping for {}. Please provide a sample to the developer on GitHub.",
                            text
                        );
                        text.as_str()
                    }
                };

                format!(
                    "<mrow><mspace width=\"0.166em\"></mspace><mi>{}</mi></mrow>",
                    text
                )
            }
            Some(TextType::Fraktur) => format!("<mi mathvariant=\"fraktur\">{}</mi>", text),
            Some(TextType::FrakturBold) => {
                format!("<mi mathvariant=\"bold-fraktur\">{}</mi>", text)
            }
            Some(TextType::Identifier) => format!("<mi>{}</mi>", text),
            // Some(TextType::Italic) => format!("<mi mathvariant=\"italic\">{}</mi>", text),
            Some(TextType::Mono) => format!("<mi mathvariant=\"monospace\">{}</mi>", text),
            Some(TextType::Normal) => format!("<mi mathvariant=\"normal\">{}</mi>", text),
            Some(TextType::Numeric) => format!("<mn>{}</mn>", text),
            Some(TextType::Operator) => format!("<mo>{}</mo>", text),
            Some(TextType::Raw) => text,
            Some(TextType::Sans) => format!("<mi mathvariant=\"sans-serif\">{}</mi>", text),
            Some(TextType::SansBold) => {
                format!("<mi mathvariant=\"sans-serif-bold\">{}</mi>", text)
            }
            Some(TextType::SansBoldItalic) => {
                format!("<mi mathvariant=\"sans-serif-bold-italic\">{}</mi>", text)
            }
            Some(TextType::SansItalic) => {
                format!("<mi mathvariant=\"sans-serif-italic\">{}</mi>", text)
            }
            Some(TextType::Script) => format!("<mi mathvariant=\"script\">{}</mi>", text),
            Some(TextType::Space) => "<mspace width=\"0.222em\"></mspace>".to_string(),
        };
        rendered_parts.push(rendered);
    }

    let text = rendered_parts.iter().join("");

    Ok(text)
}

fn is_skippable_format(c: char) -> bool {
    c.is_format() && c != '\u{2061}'
}

fn render_accent(char: char, body: Equation) -> Result<String> {
    Ok(format!(
        "<mover accent=\"true\">{}<mo>{}</mo></mover>",
        render_group(body)?,
        char
    ))
}

fn render_box(body: Equation, align: Option<BoxDisplay>) -> Result<String> {
    let Some(display) = align else {
        return render_group(body);
    };

    let mut content = render_group(body)?;

    if display.align != BoxAlignment::Baseline {
        content = format!("<mrow><malignmark/>{}</mrow>", content);
    }

    let (lspace, rspace) = match display.space {
        BoxSpace::Unary => (None, Some("0.166em")),
        BoxSpace::Binary => (Some("0.222em"), Some("0.222em")),
        BoxSpace::Relational => (Some("0.278em"), Some("0.278em")),
        BoxSpace::Skip => (Some("0.444em"), Some("0.444em")),
        BoxSpace::Ord => (None, None),
        BoxSpace::Differential => (Some("0.111em"), None),
        _ => (None, None),
    };

    if lspace.is_some() || rspace.is_some() {
        let left = lspace
            .map(|width| format!("<mspace width=\"{}\"></mspace>", width))
            .unwrap_or_default();
        let right = rspace
            .map(|width| format!("<mspace width=\"{}\"></mspace>", width))
            .unwrap_or_default();
        content = format!("<mrow>{}{}{}</mrow>", left, content, right);
    }

    content = match display.size {
        BoxSize::Script => format!(
            "<mstyle scriptlevel=\"1\" displaystyle=\"false\">{}</mstyle>",
            content
        ),
        BoxSize::ScriptScript => format!(
            "<mstyle scriptlevel=\"2\" displaystyle=\"false\">{}</mstyle>",
            content
        ),
        _ => content,
    };

    if display.flags.contains(BoxFlags::NoBreak) {
        content = format!("<mrow linebreak=\"nobreak\">{}</mrow>", content);
    }

    Ok(content)
}

fn render_boxed_formula(body: Equation, align: Option<BoxedFormulaAlignment>) -> Result<String> {
    if align.is_some() {
        warn!(
            "Math feature not implemented: boxed-formula alignment. Please provide a sample at https://github.com/msiemens/one2html/issues."
        );
    }

    Ok(format!(
        "<menclose notation=\"box\">{}</menclose>",
        render_group(body)?
    ))
}

fn render_brackets(
    open: Option<char>,
    close: Option<char>,
    body: Equation,
    align: Option<BracketsAlignment>,
) -> Result<String> {
    let size = bracket_size(align);

    let open = open
        .map(|c| format!("<mo{}>{}</mo>", size, c))
        .unwrap_or_default();
    let close = close
        .map(|c| format!("<mo{}>{}</mo>", size, c))
        .unwrap_or_default();

    Ok(format!(
        "<mrow>{}{}{}</mrow>",
        open,
        render_group(body)?,
        close
    ))
}

fn bracket_size(align: Option<BracketsAlignment>) -> String {
    align
        .map(|align| match align {
            BracketsAlignment::DontGrow => brackets_size(0),
            BracketsAlignment::TeXbig => brackets_size(1),
            BracketsAlignment::TeXBig => brackets_size(2),
            BracketsAlignment::TeXbigg => brackets_size(3),
            BracketsAlignment::TeXBigg => brackets_size(4),
        })
        .map(|size| format!(" minsize=\"{0}\" maxsize=\"{0}\"", size))
        .unwrap_or_default()
}

fn brackets_size(size: i32) -> String {
    format!("{}em", 1.25f32.powi(size))
}

fn render_brackets_with_seps(
    open: Option<char>,
    close: Option<char>,
    sep: char,
    segments: Vec<Equation>,
    align: Option<BracketsAlignment>,
) -> Result<String> {
    let size = bracket_size(align);

    let open = open
        .map(|c| format!("<mo symmetric=\"true\"{}>{}</mo>", size, c))
        .unwrap_or_default();
    let close = close
        .map(|c| format!("<mo symmetric=\"true\"{}>{}</mo>", size, c))
        .unwrap_or_default();
    let sep = format!(
        "<mo stretchy=\"true\" symmetric=\"true\"{}>{}</mo>",
        size, sep
    );
    let content = segments
        .into_iter()
        .map(render_group)
        .collect::<Result<Vec<_>>>()?
        .join(&sep);

    Ok(format!("<mrow>{}{}{}</mrow>", open, content, close))
}

fn render_equation_array(
    _columns: u8,
    rows: Vec<Equation>,
    align: Option<EquationArrayAlignment>,
) -> Result<String> {
    if align.is_some() {
        warn!(
            "Math feature not implemented: equation-array alignment. Please provide a sample to the developer on GitHub."
        );
    }

    // FIXME: Unused columns?
    // FIXME: Handle escaped &?

    let rows = rows
        .into_iter()
        .map(|row| Ok(format!("<mtr><mtd>{}</mtd></mtr>", render_group(row)?)))
        .collect::<Result<Vec<_>>>()?
        .join("")
        .replace('&', "<malignmark edge=\"left\"></malignmark>");

    Ok(format!("<mtable>{}</mtable>", rows))
}

fn render_fraction(num: Equation, den: Equation, small: bool) -> Result<String> {
    if small {
        return Ok(format!(
            "<mfrac>{}{}</mfrac>",
            render_group(num)?,
            render_group(den)?
        ));
    }

    Ok(format!(
        "<mfrac>{}{}</mfrac>",
        render_group(num)?,
        render_group(den)?
    ))
}

fn render_function_apply(func: Equation, body: Equation) -> Result<String> {
    Ok(format!(
        "{}<mo>\u{2061}</mo>{}",
        render_group(func)?,
        render_group(body)?
    ))
}

fn render_left_sub_sup(sub: Equation, sup: Equation, body: Equation) -> Result<String> {
    let sup = if sup.is_empty() {
        "<none/>".to_string()
    } else {
        render_group(sup)?
    };
    let sub = if sub.is_empty() {
        "<none/>".to_string()
    } else {
        render_group(sub)?
    };

    Ok(format!(
        "<mmultiscripts>{}<none/><none/><mprescripts/>{}{}</mmultiscripts>",
        render_group(body)?,
        sub,
        sup,
    ))
}

fn render_lower_limit(body: Equation, limit: Equation) -> Result<String> {
    Ok(format!(
        "<munder>{}{}</munder>",
        render_group(body)?,
        render_group(limit)?,
    ))
}

fn render_matrix(
    columns: u8,
    items: Vec<Equation>,
    brackets: Option<MatrixBrackets>,
    align: Option<MatrixAlignment>,
) -> Result<String> {
    let show_placeholder = matches!(align, Some(MatrixAlignment::ShowMatPlaceHldr));
    let row_align = match align {
        Some(MatrixAlignment::MatrixAlignTopRow) => " rowalign=\"top\"",
        Some(MatrixAlignment::MatrixAlignBottomRow) => " rowalign=\"bottom\"",
        _ => "",
    };

    let (open, close) = brackets.map(render_matrix_brackets).unwrap_or_default();
    let rows = items
        .into_iter()
        .chunks(columns as usize)
        .into_iter()
        .map(|row| {
            let mut cells = row.collect_vec();
            while cells.len() < columns as usize {
                cells.push(Vec::new());
            }
            Ok(format!(
                "<mtr>{}</mtr>",
                render_matrix_row(cells, show_placeholder)?
            ))
        })
        .collect::<Result<Vec<_>>>()?
        .join("");

    Ok(format!(
        "<mrow><mo>{}</mo><mtable columnspacing=\"0.8em\"{}>{}</mtable><mo>{}</mo></mrow>",
        open, row_align, rows, close
    ))
}

fn render_matrix_row(items: Vec<Equation>, show_placeholder: bool) -> Result<String> {
    items
        .into_iter()
        .map(|eq| {
            if eq.is_empty() && show_placeholder {
                Ok("<mtd><mo>□</mo></mtd>".to_string())
            } else if eq.is_empty() {
                Ok("<mtd></mtd>".to_string())
            } else {
                Ok(format!("<mtd>{}</mtd>", render_group(eq)?))
            }
        })
        .collect::<Result<Vec<_>>>()
        .map(|cells| cells.join(""))
}

fn render_matrix_brackets(brackets: MatrixBrackets) -> (char, char) {
    match brackets {
        MatrixBrackets::Parentheses => ('(', ')'),
        MatrixBrackets::VerticalBars => ('|', '|'),
        MatrixBrackets::DoubleVerticalBars => ('‖', '‖'),
    }
}

fn render_nary(
    op: char,
    sub: Equation,
    sup: Equation,
    body: Equation,
    display: Option<NAryDisplay>,
) -> Result<String> {
    let sub_is_empty = sub.is_empty();
    let sup_is_empty = sup.is_empty();
    let mut op_node = format!("<mo>{}</mo>", op);

    if let Some(display) = display {
        if let Some(flags) = display.flags {
            if flags == NAryFlags::DontGrowWithContent {
                op_node = format!("<mo stretchy=\"false\">{}</mo>", op);
            } else if flags == NAryFlags::GrowWithContent {
                op_node = format!("<mo stretchy=\"true\">{}</mo>", op);
            }
        }

        let mut sub_content = render_group(sub)?;
        let mut sup_content = render_group(sup)?;

        if display.options.contains(NAryOptions::ShowLLimPlaceHldr) && sub_is_empty {
            sub_content = "<mrow><mo>□</mo></mrow>".to_string();
        }

        if display.options.contains(NAryOptions::ShowULimPlaceHldr) && sup_is_empty {
            sup_content = "<mrow><mo>□</mo></mrow>".to_string();
        }

        if display.options.contains(NAryOptions::LimitsOpposite) {
            std::mem::swap(&mut sub_content, &mut sup_content);
        }

        let nary = match display.align {
            NAryAlignment::LimitsSubSup | NAryAlignment::UpperLimitAsSuperScript => {
                format!(
                    "<msubsup>{}{}{}</msubsup>",
                    op_node, sub_content, sup_content
                )
            }
            _ => format!(
                "<munderover>{}{}{}</munderover>",
                op_node, sub_content, sup_content
            ),
        };

        return Ok(format!("<mrow>{}{}</mrow>", nary, render_group(body)?));
    }

    let sub = render_group(sub)?;
    let sup = render_group(sup)?;
    Ok(format!(
        "<mrow><munderover>{}{}{}</munderover>{}</mrow>",
        op_node,
        sub,
        sup,
        render_group(body)?
    ))
}

fn render_over_bar(body: Equation) -> Result<String> {
    Ok(format!(
        "<mover accent=\"true\">{}<mo>\u{00af}</mo></mover>",
        render_group(body)?
    ))
}

fn render_phantom(
    kind: PhantomKind,
    display: Option<PhantomDisplay>,
    body: Equation,
) -> Result<String> {
    let mut show = matches!(
        kind,
        PhantomKind::AscentSmash
            | PhantomKind::DescentSmash
            | PhantomKind::HorizontalSmash
            | PhantomKind::VerticalSmash
    );
    let mut transparent = false;
    let mut zero_width = false;
    let mut zero_ascent = false;
    let mut zero_descent = false;

    if let Some(display) = display {
        show |= display.contains(PhantomDisplay::PhantomShow);
        transparent |= display.contains(PhantomDisplay::PhantomTransparent);
        zero_width |= display.contains(PhantomDisplay::PhantomZeroWidth);
        zero_ascent |= display.contains(PhantomDisplay::PhantomZeroAscent);
        zero_descent |= display.contains(PhantomDisplay::PhantomZeroDescent);
    }

    match kind {
        PhantomKind::FullOrCustom => {}
        PhantomKind::HorizontalPhantom => {
            zero_ascent = true;
            zero_descent = true;
        }
        PhantomKind::VerticalPhantom => {
            zero_width = true;
        }
        PhantomKind::AscentSmash => {
            zero_ascent = true;
        }
        PhantomKind::DescentSmash => {
            zero_descent = true;
        }
        PhantomKind::HorizontalSmash => {
            zero_width = true;
        }
        PhantomKind::VerticalSmash => {
            zero_ascent = true;
            zero_descent = true;
        }
    }

    let mut content = render_group(body)?;

    if !show {
        content = format!("<mphantom>{}</mphantom>", content);
    } else if transparent {
        content = format!("<mstyle mathcolor=\"transparent\">{}</mstyle>", content);
    }

    if zero_width || zero_ascent || zero_descent {
        let mut attrs = String::new();
        if zero_width {
            attrs.push_str(" width=\"0\"");
        }
        if zero_ascent {
            attrs.push_str(" height=\"0\"");
        }
        if zero_descent {
            attrs.push_str(" depth=\"0\"");
        }

        content = format!("<mpadded{}>{}</mpadded>", attrs, content);
    }

    Ok(content)
}

fn render_radical(degree: Equation, body: Equation) -> Result<String> {
    Ok(format!(
        "<mroot>{}{}</mroot>",
        render_group(body)?,
        render_group(degree)?
    ))
}

fn render_slashed_fraction(num: Equation, den: Equation, linear: bool) -> Result<String> {
    if linear {
        return Ok(format!(
            "{}<mo>⁄</mo>{}",
            render_group(num)?,
            render_group(den)?
        ));
    }

    let num = render_group(num)?;
    let den = render_group(den)?;

    Ok(format!(
        "<mrow><msup><mrow/><mstyle scriptlevel=\"1\" displaystyle=\"false\">{}</mstyle></msup><mo>⁄</mo><msub><mrow/><mstyle scriptlevel=\"1\" displaystyle=\"false\">{}</mstyle></msub></mrow>",
        num, den
    ))
}

fn render_stack(num: Equation, den: Equation) -> Result<String> {
    Ok(format!(
        "<mtable><mtr><mtd>{}</mtd></mtr><mtr><mtd>{}</mtd></mtr></mtable>",
        render_group(num)?,
        render_group(den)?
    ))
}

fn render_stretch_stack(char: char, pos: StretchStackPosition, body: Equation) -> Result<String> {
    // OneNote distinguishes "char vs base" above/below, but MathML mover/munder
    // always use the first child as the base and the second as the stacked mark,
    // so we always treat `body` as the base and `char` as the stacked element.

    let mtype = match pos {
        StretchStackPosition::CharBelow => "munder",
        StretchStackPosition::CharAbove => "mover",
        StretchStackPosition::BaseBelow => "mover",
        StretchStackPosition::BaseAbove => "munder",
    };

    Ok(format!(
        "<{} accent=\"true\">{}<mo>{}</mo></{}>",
        mtype,
        render_group(body)?,
        char,
        mtype,
    ))
}

fn render_subscript(sub: Equation, body: Equation) -> Result<String> {
    let sub = if sub.is_empty() {
        "<mo>⬚</mo>".to_string()
    } else {
        render_group(sub)?
    };

    Ok(format!("<msub>{}{}</msub>", render_group(body)?, sub))
}

fn render_sub_sup(
    sub: Equation,
    sup: Equation,
    body: Equation,
    align: Option<SubSupAlignment>,
) -> Result<String> {
    if align.is_some() {
        warn!(
            "Math feature not implemented: sub-sup alignment. Please provide a sample to the developer on GitHub."
        );
    }

    let sub = if sub.is_empty() {
        "<mo>⬚</mo>".to_string()
    } else {
        render_group(sub)?
    };
    let sup = if sup.is_empty() {
        "<mo>⬚</mo>".to_string()
    } else {
        render_group(sup)?
    };

    Ok(format!(
        "<msubsup>{}{}{}</msubsup>",
        render_group(body)?,
        sub,
        sup,
    ))
}

fn render_superscript(sup: Equation, body: Equation) -> Result<String> {
    let sup = if sup.is_empty() {
        "<mo>⬚</mo>".to_string()
    } else {
        render_group(sup)?
    };

    Ok(format!("<msup>{}{}</msup>", render_group(body)?, sup))
}

fn render_under_bar(body: Equation) -> Result<String> {
    Ok(format!(
        "<munder accentunder=\"true\">{}<mo>_</mo></munder>",
        render_group(body)?
    ))
}

fn render_upper_limit(body: Equation, limit: Equation) -> Result<String> {
    Ok(format!(
        "<mover>{}{}</mover>",
        render_group(body)?,
        render_group(limit)?,
    ))
}
