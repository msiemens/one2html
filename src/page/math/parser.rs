use super::ast::Equation;
use crate::page::math::ast::{
    BoxAlignment, BoxDisplay, BoxFlags, BoxSize, BoxSpace, BoxedFormulaAlignment,
    BracketsAlignment, EquationArrayAlignment, MathOp, MatrixAlignment, MatrixBrackets,
    NAryAlignment, NAryDisplay, NAryFlags, NAryOptions, PhantomDisplay, PhantomKind,
    StretchStackPosition, SubSupAlignment,
};
use crate::page::math::lexer::{Lexer, Token};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use log::warn;
use num_traits::FromPrimitive;
use onenote_parser::contents::{MathInlineObject, MathObjectType};

pub(super) struct Parser {
    lexer: Lexer,
    cur: Token,
}

impl Parser {
    pub(super) fn new(tokens: Vec<(String, MathInlineObject)>) -> Result<Parser> {
        let mut lexer = Lexer::new(tokens);
        let cur = lexer.next()?;

        Ok(Parser { lexer, cur })
    }

    pub(super) fn parse(&mut self) -> Result<Equation> {
        let mut equation = Vec::new();

        while self.cur != Token::Eof {
            let math_op = self.parse_op()?;
            equation.push(math_op);
        }

        Ok(equation)
    }

    fn parse_op(&mut self) -> Result<MathOp> {
        match &self.cur {
            Token::Text(_) => self.parse_text(),
            Token::Start(object) => match object.object_type() {
                MathObjectType::Accent => self.parse_accent(),
                MathObjectType::Box => self.parse_box(),
                MathObjectType::BoxedFormula => self.parse_boxed_formula(),
                MathObjectType::Brackets => self.parse_brackets(),
                MathObjectType::BracketsWithSeps => self.parse_brackets_with_seps(),
                MathObjectType::EquationArray => self.parse_equation_array(),
                MathObjectType::Fraction => self.parse_fraction(),
                MathObjectType::FunctionApply => self.parse_function_apply(),
                MathObjectType::LeftSubSup => self.parse_left_sub_sup(),
                MathObjectType::LowerLimit => self.parse_lower_limit(),
                MathObjectType::Matrix => self.parse_matrix(),
                MathObjectType::Nary => self.parse_nary(),
                MathObjectType::OpChar => self.parse_op_char(),
                MathObjectType::Overbar => self.parse_over_bar(),
                MathObjectType::Phantom => self.parse_phantom(),
                MathObjectType::Radical => self.parse_radical(),
                MathObjectType::SlashedFraction => self.parse_slashed_fraction(),
                MathObjectType::Stack => self.parse_stack(),
                MathObjectType::StretchStack => self.parse_stretch_stack(),
                MathObjectType::Subscript => self.parse_subscript(),
                MathObjectType::SubSup => self.parse_sub_sup(),
                MathObjectType::Superscript => self.parse_superscript(),
                MathObjectType::Underbar => self.parse_under_bar(),
                MathObjectType::UpperLimit => self.parse_upper_limit(),
                _ => Err(eyre!(
                    "Unexpected math object start: {:?} (SimpleText or PlainText not tokenized into Token::Text)",
                    object.object_type()
                )),
            },
            Token::Sep(_) => Err(eyre!(
                "Unexpected math object separator (expected text or math object start)"
            )),
            Token::End(_) => Err(eyre!(
                "Unexpected math object end (expected text or math object start)"
            )),
            Token::Eof => Err(eyre!(
                "Unexpected math object EOF (expected text or math object start)"
            )),
        }
    }

    fn parse_text(&mut self) -> Result<MathOp> {
        match self.bump()? {
            Token::Text(text) => Ok(MathOp::Text(text)),
            tok => Err(eyre!("Unexpected math token: {:?} (expected text)", tok)),
        }
    }

    fn parse_accent(&mut self) -> Result<MathOp> {
        let (object, body) = self.parse_object_1(MathObjectType::Accent)?;

        self.expect_no_align_data(&object)?;

        let char = object
            .char()
            .ok_or_else(|| eyre!("No accent char has been defined"))?;

        Ok(MathOp::Accent { char, body })
    }

    fn parse_box(&mut self) -> Result<MathOp> {
        let (object, body) = self.parse_object_1(MathObjectType::Box)?;

        let display = object.align().map(|value| {
            // `value` is a packed bitfield with non-overlapping fields; each type masks its own
            // bits via from_bits_truncate.
            let align = BoxAlignment::from_bits_truncate(value);
            let space = BoxSpace::from_bits_truncate(value);
            let size = BoxSize::from_bits_truncate(value);
            let flags = BoxFlags::from_bits_truncate(value);

            BoxDisplay {
                align,
                space,
                size,
                flags,
            }
        });

        Ok(MathOp::Box { body, display })
    }

    fn parse_boxed_formula(&mut self) -> Result<MathOp> {
        let (object, body) = self.parse_object_1(MathObjectType::BoxedFormula)?;

        let align = object
            .align()
            .map(|value| {
                BoxedFormulaAlignment::from_bits(value)
                    .ok_or_else(|| eyre!("Invalid boxed formula alignment: {}", value))
            })
            .transpose()?;

        Ok(MathOp::BoxedFormula { align, body })
    }

    fn parse_brackets(&mut self) -> Result<MathOp> {
        let (object, body) = self.parse_object_1(MathObjectType::Brackets)?;

        let align = object.align().and_then(BracketsAlignment::from_u8);

        let open = object.char();
        let close = object.char1();

        Ok(MathOp::Brackets {
            align,
            open,
            close,
            body,
        })
    }

    fn parse_brackets_with_seps(&mut self) -> Result<MathOp> {
        let (object, segments) = self.parse_object_n(MathObjectType::BracketsWithSeps)?;

        let align = object.align().and_then(BracketsAlignment::from_u8);

        let open = object.char();
        let close = object.char1();
        let sep = object
            .char2()
            .ok_or_else(|| eyre!("Brackets with seps has no separator character"))?;

        Ok(MathOp::BracketsWithSeps {
            align,
            open,
            close,
            sep,
            segments,
        })
    }

    fn parse_equation_array(&mut self) -> Result<MathOp> {
        let (object, rows) = self.parse_object_n(MathObjectType::EquationArray)?;

        let columns = object
            .column()
            .ok_or_else(|| eyre!("Equation array columns are not set"))?;

        let align = object
            .align()
            .map(|value| {
                EquationArrayAlignment::from_u8(value)
                    .ok_or_else(|| eyre!("Invalid equation array alignment: {}", value))
            })
            .transpose()?;

        Ok(MathOp::EquationArray {
            align,
            columns,
            rows,
        })
    }

    fn parse_fraction(&mut self) -> Result<MathOp> {
        let (object, num, den) = self.parse_object_2(MathObjectType::Fraction)?;

        let small = object.char() == Some('\u{2298}');

        self.expect_no_align_data(&object)?;

        Ok(MathOp::Fraction { num, den, small })
    }

    fn parse_function_apply(&mut self) -> Result<MathOp> {
        let (object, func, body) = self.parse_object_2(MathObjectType::FunctionApply)?;

        self.expect_no_align_data(&object)?;

        Ok(MathOp::FunctionApply { func, body })
    }

    fn parse_left_sub_sup(&mut self) -> Result<MathOp> {
        let (object, sub, sup, body) = self.parse_object_3(MathObjectType::LeftSubSup)?;

        self.expect_no_align_data(&object)?;

        Ok(MathOp::LeftSubSup { sub, sup, body })
    }

    fn parse_lower_limit(&mut self) -> Result<MathOp> {
        let (object, body, limit) = self.parse_object_2(MathObjectType::LowerLimit)?;

        self.expect_no_align_data(&object)?;

        Ok(MathOp::LowerLimit { body, limit })
    }

    fn parse_matrix(&mut self) -> Result<MathOp> {
        // TODO: Parse other brackets
        let (object, items) = self.parse_object_n(MathObjectType::Matrix)?;

        let columns = object
            .column()
            .ok_or_else(|| eyre!("Equation array columns are not set"))?;

        let char = object
            .char()
            .ok_or_else(|| eyre!("Matrix has no brackets specifier"))?;
        let brackets = match char {
            '\u{25a0}' => None,
            '\u{24a8}' => Some(MatrixBrackets::Parentheses),
            '\u{24b1}' => Some(MatrixBrackets::VerticalBars),
            '\u{24a9}' => Some(MatrixBrackets::DoubleVerticalBars),
            c => return Err(eyre!("Invalid matrix brackets specifier: {}", c)),
        };

        let align = object
            .align()
            .map(|value| {
                MatrixAlignment::from_u8(value)
                    .ok_or_else(|| eyre!("Invalid matrix alignment: {}", value))
            })
            .transpose()?;

        Ok(MathOp::Matrix {
            align,
            columns,
            brackets,
            items,
        })
    }

    fn parse_nary(&mut self) -> Result<MathOp> {
        let (object, sub, sup, body) = self.parse_object_3(MathObjectType::Nary)?;

        let op = object
            .char()
            .ok_or_else(|| eyre!("N-ary has no operator char"))?;

        let display = object
            .align()
            .map::<Result<_>, _>(|value| {
                let align = NAryAlignment::from_u8(value & 0x3)
                    .ok_or_else(|| eyre!("Invalid n-ary alignment: {}", value))?;
                let options = NAryOptions::from_bits_truncate(value);
                let flags = NAryFlags::from_u8(value & 0xc0);

                Ok(NAryDisplay {
                    align,
                    options,
                    flags,
                })
            })
            .transpose()?;

        Ok(MathOp::NAry {
            op,
            display,
            sub,
            sup,
            body,
        })
    }

    fn parse_op_char(&mut self) -> Result<MathOp> {
        let object = self.bump_start(MathObjectType::OpChar)?;
        self.expect_argc(&object, 0)?;
        self.bump_end(MathObjectType::OpChar)?;

        let text = object
            .char()
            .map(|value| value.to_string())
            .unwrap_or_default();

        warn!(
            "Math feature not implemented: op-char handling. Please provide a sample to the developer on GitHub."
        );

        Ok(MathOp::Text(text))
    }

    fn parse_over_bar(&mut self) -> Result<MathOp> {
        let (object, body) = self.parse_object_1(MathObjectType::Overbar)?;

        self.expect_no_align_data(&object)?;

        Ok(MathOp::OverBar { body })
    }

    fn parse_phantom(&mut self) -> Result<MathOp> {
        let (object, body) = self.parse_object_1(MathObjectType::Phantom)?;

        let char = object
            .char()
            .ok_or_else(|| eyre!("Phantom has no kind specifier"))?;
        let kind = match char {
            '\u{27e1}' => PhantomKind::FullOrCustom,
            '\u{2b04}' => PhantomKind::HorizontalPhantom,
            '\u{21f3}' => PhantomKind::VerticalPhantom,
            '\u{2b06}' => PhantomKind::AscentSmash,
            '\u{2b07}' => PhantomKind::DescentSmash,
            '\u{2b0c}' => PhantomKind::HorizontalSmash,
            '\u{2b0d}' => PhantomKind::VerticalSmash,
            c => return Err(eyre!("Invalid phantom kind: {}", c)),
        };

        let display = object
            .align()
            .map(|value| {
                PhantomDisplay::from_bits(value)
                    .ok_or_else(|| eyre!("Invalid phantom display: {}", value))
            })
            .transpose()?;

        Ok(MathOp::Phantom {
            body,
            kind,
            display,
        })
    }

    fn parse_radical(&mut self) -> Result<MathOp> {
        let (object, degree, body) = self.parse_object_2(MathObjectType::Radical)?;

        self.expect_no_align_data(&object)?;

        Ok(MathOp::Radical {
            // display: None,
            degree,
            body,
        })
    }

    fn parse_slashed_fraction(&mut self) -> Result<MathOp> {
        let (object, num, den) = self.parse_object_2(MathObjectType::SlashedFraction)?;

        let linear = object.char() == Some('\u{2215}');

        self.expect_no_align_data(&object)?;

        Ok(MathOp::SlashedFraction { num, den, linear })
    }

    fn parse_stack(&mut self) -> Result<MathOp> {
        let (object, num, den) = self.parse_object_2(MathObjectType::Stack)?;

        self.expect_no_align_data(&object)?;

        Ok(MathOp::Stack { num, den })
    }

    fn parse_stretch_stack(&mut self) -> Result<MathOp> {
        let (object, body) = self.parse_object_1(MathObjectType::StretchStack)?;

        let char = object
            .char()
            .ok_or_else(|| eyre!("No stretch char has been defined"))?;

        let align = object.align().unwrap_or(0);
        let pos = StretchStackPosition::from_u8(align)
            .ok_or_else(|| eyre!("Invalid stretch position: {}", align))?;

        Ok(MathOp::StretchStack { char, body, pos })
    }

    fn parse_subscript(&mut self) -> Result<MathOp> {
        let (object, body, sub) = self.parse_object_2(MathObjectType::Subscript)?;

        self.expect_no_align_data(&object)?;

        Ok(MathOp::Subscript { sub, body })
    }

    fn parse_sub_sup(&mut self) -> Result<MathOp> {
        let (object, body, sub, sup) = self.parse_object_3(MathObjectType::SubSup)?;

        let align = object
            .align()
            .map(|value| {
                SubSupAlignment::from_u8(value)
                    .ok_or_else(|| eyre!("Invalid sub sup alignment: {}", value))
            })
            .transpose()?;

        Ok(MathOp::SubSup {
            align,
            sub,
            sup,
            body,
        })
    }

    fn parse_superscript(&mut self) -> Result<MathOp> {
        let (object, body, sup) = self.parse_object_2(MathObjectType::Superscript)?;

        self.expect_no_align_data(&object)?;

        Ok(MathOp::Superscript { sup, body })
    }

    fn parse_under_bar(&mut self) -> Result<MathOp> {
        let (object, body) = self.parse_object_1(MathObjectType::Underbar)?;

        self.expect_no_align_data(&object)?;

        Ok(MathOp::UnderBar { body })
    }

    fn parse_upper_limit(&mut self) -> Result<MathOp> {
        let (object, body, limit) = self.parse_object_2(MathObjectType::UpperLimit)?;

        self.expect_no_align_data(&object)?;

        Ok(MathOp::UpperLimit { body, limit })
    }

    // Helpers
    fn parse_object_1(
        &mut self,
        object_type: MathObjectType,
    ) -> Result<(MathInlineObject, Equation)> {
        let object = self.bump_start(object_type)?;
        self.expect_argc(&object, 1)?;

        let arg = self.parse_arg(object_type)?;
        self.bump_end(object_type)?;

        Ok((object, arg))
    }

    fn parse_object_2(
        &mut self,
        object_type: MathObjectType,
    ) -> Result<(MathInlineObject, Equation, Equation)> {
        let object = self.bump_start(object_type)?;
        self.expect_argc(&object, 2)?;

        let arg1 = self.parse_arg(object_type)?;
        self.bump_sep(object_type)?;

        let arg2 = self.parse_arg(object_type)?;
        self.bump_end(object_type)?;

        Ok((object, arg1, arg2))
    }

    fn parse_object_3(
        &mut self,
        object_type: MathObjectType,
    ) -> Result<(MathInlineObject, Equation, Equation, Equation)> {
        let object = self.bump_start(object_type)?;
        self.expect_argc(&object, 3)?;

        let arg1 = self.parse_arg(object_type)?;
        self.bump_sep(object_type)?;

        let arg2 = self.parse_arg(object_type)?;
        self.bump_sep(object_type)?;

        let arg3 = self.parse_arg(object_type)?;
        self.bump_end(object_type)?;

        Ok((object, arg1, arg2, arg3))
    }

    fn parse_object_n(
        &mut self,
        object_type: MathObjectType,
    ) -> Result<(MathInlineObject, Vec<Equation>)> {
        let object = self.bump_start(object_type)?;

        let n = object.arg_count();
        let args = (0..n)
            .map(|i| {
                let arg = self.parse_arg(object_type)?;

                if i < n - 1 {
                    self.bump_sep(object_type)?;
                } else {
                    self.bump_end(object_type)?;
                }

                Ok(arg)
            })
            .collect::<Result<_>>()?;

        Ok((object, args))
    }

    fn parse_arg(&mut self, object_type: MathObjectType) -> Result<Equation> {
        let mut equation = Vec::new();

        while self.cur != Token::Sep(object_type) && self.cur != Token::End(object_type) {
            let math_op = self.parse_op()?;
            equation.push(math_op);
        }

        Ok(equation)
    }

    fn bump(&mut self) -> Result<Token> {
        let token = std::mem::replace(&mut self.cur, self.lexer.next()?);

        Ok(token)
    }

    fn bump_start(&mut self, object_type: MathObjectType) -> Result<MathInlineObject> {
        let token = self.bump()?;
        match token {
            Token::Start(object) if object.object_type() == object_type => Ok(object),
            tok => Err(eyre!(
                "Unexpected math token: {:?} (expected math object start {:?})",
                tok,
                object_type
            )),
        }
    }

    fn bump_sep(&mut self, object_type: MathObjectType) -> Result<()> {
        let token = self.bump()?;
        match token {
            Token::Sep(tok_object_type) if tok_object_type == object_type => Ok(()),
            tok => Err(eyre!(
                "Unexpected math token: {:?} (expected math object end {:?})",
                tok,
                object_type
            )),
        }
    }

    fn bump_end(&mut self, object_type: MathObjectType) -> Result<()> {
        let token = self.bump()?;
        match token {
            Token::End(tok_object_type) if tok_object_type == object_type => Ok(()),
            tok => Err(eyre!(
                "Unexpected math token: {:?} (expected math object end {:?})",
                tok,
                object_type
            )),
        }
    }

    fn expect_argc(&self, object: &MathInlineObject, count: u32) -> Result<()> {
        if object.arg_count() != count {
            return Err(eyre!(
                "Unexpected argument count: {} (expected {})",
                object.arg_count(),
                count
            ));
        }

        Ok(())
    }

    fn expect_no_align_data(&self, object: &MathInlineObject) -> Result<()> {
        if object.align().is_some() {
            return Err(eyre!(
                "Unexpected math object align (type: {:?})",
                MathObjectType::FunctionApply
            ));
        }

        Ok(())
    }
}
