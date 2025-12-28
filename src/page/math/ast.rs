use bitflags::bitflags;
use enum_primitive_derive::Primitive;

pub(super) type Equation = Vec<MathOp>;

#[derive(Debug)]
pub(super) enum MathOp {
    Text(String),
    Accent {
        char: char,
        body: Equation,
    },
    Box {
        // kind: char,
        body: Equation,
        display: Option<BoxDisplay>,
    },
    BoxedFormula {
        // kind: char,
        body: Equation,
        align: Option<BoxedFormulaAlignment>,
    },
    Brackets {
        open: Option<char>,
        close: Option<char>,
        body: Equation,
        align: Option<BracketsAlignment>,
    },
    BracketsWithSeps {
        open: Option<char>,
        close: Option<char>,
        sep: char,
        segments: Vec<Equation>,
        align: Option<BracketsAlignment>,
    },
    EquationArray {
        columns: u8,
        rows: Vec<Equation>,
        align: Option<EquationArrayAlignment>,
    },
    Fraction {
        num: Equation,
        den: Equation,
        small: bool,
    },
    FunctionApply {
        func: Equation,
        body: Equation,
    },
    LeftSubSup {
        sub: Equation,
        sup: Equation,
        body: Equation,
    },
    LowerLimit {
        body: Equation,
        limit: Equation,
    },
    Matrix {
        columns: u8,
        brackets: Option<MatrixBrackets>,
        items: Vec<Equation>,
        align: Option<MatrixAlignment>,
    },
    NAry {
        op: char,
        sub: Equation,
        sup: Equation,
        body: Equation,
        display: Option<NAryDisplay>,
    },
    // OpChar(char),
    OverBar {
        body: Equation,
    },
    Phantom {
        body: Equation,
        kind: PhantomKind,
        display: Option<PhantomDisplay>,
    },
    Radical {
        body: Equation,
        degree: Equation,
        // display: Option<RadicalDisplay>,
    },
    SlashedFraction {
        num: Equation,
        den: Equation,
        linear: bool,
    },
    Stack {
        num: Equation,
        den: Equation,
    },
    StretchStack {
        char: char,
        body: Equation,
        pos: StretchStackPosition,
    },
    Subscript {
        sub: Equation,
        body: Equation,
    },
    SubSup {
        sub: Equation,
        sup: Equation,
        body: Equation,
        align: Option<SubSupAlignment>,
    },
    Superscript {
        sup: Equation,
        body: Equation,
    },
    UnderBar {
        body: Equation,
    },
    UpperLimit {
        body: Equation,
        limit: Equation,
    },
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(super) struct BoxDisplay {
    pub(crate) align: BoxAlignment,
    pub(crate) space: BoxSpace,
    pub(crate) size: BoxSize,
    pub(crate) flags: BoxFlags,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(super) struct BoxAlignment: u8 {
        const Baseline = 0;
        const Center = 1;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(super) struct BoxSpace: u8 {
        // const SpaceMask = 0;
        const Default = 0;
        const Unary = 4;
        const Binary = 8;
        const Relational = 12;
        const Skip = 16;
        const Ord = 20;
        const Differential = 24;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(super) struct BoxSize: u8 {
        const Text = 0;
        const Script = 32;
        const ScriptScript = 64;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub(super) struct BoxedFormulaAlignment: u8 {
        const BoxHideTop = 1;
        const BoxHideBottom = 2;
        const BoxHideLeft = 4;
        const BoxHideRight = 8;
        const BoxStrikeH = 16;
        const BoxStrikeV = 32;
        const BoxStrikeTLBR = 64;
        const BoxStrikeBLTR = 128;
    }
}

#[derive(Debug, Copy, Clone, Primitive)]
pub(super) enum BracketsAlignment {
    // AlignCenter,
    // AlignMatchAscentDescent,
    // MathVariant,
    DontGrow = 64,
    TeXbig = 32,
    TeXBig = 96,
    TeXbigg = 160,
    TeXBigg = 224,
}

#[derive(Debug, Copy, Clone, Primitive)]
#[allow(clippy::enum_variant_names)]
pub(super) enum EquationArrayAlignment {
    EqArrayLayoutWidth = 0,
    EqArrayAlignTopRow = 4,
    EqArrayAlignBottomRow = 12,
}

#[derive(Debug, Copy, Clone, Primitive)]
pub(super) enum MatrixAlignment {
    MatrixAlignCenter = 0,
    MatrixAlignTopRow = 1,
    MatrixAlignBottomRow = 3,
    ShowMatPlaceHldr = 8,
}

#[derive(Debug, Copy, Clone)]
pub(super) enum MatrixBrackets {
    Parentheses,
    VerticalBars,
    DoubleVerticalBars,
}

#[derive(Debug, Copy, Clone)]
pub(super) struct NAryDisplay {
    pub(super) align: NAryAlignment,
    pub(super) options: NAryOptions,
    pub(super) flags: Option<NAryFlags>,
}

#[derive(Debug, Copy, Clone, Primitive)]
pub(super) enum NAryAlignment {
    LimitsDefault = 0,
    LimitsUnderOver = 1,
    LimitsSubSup = 2,
    UpperLimitAsSuperScript = 3,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub(super) struct NAryOptions: u8 {
        const LimitsOpposite = 4;
        const ShowLLimPlaceHldr = 8;
        const ShowULimPlaceHldr = 16;
    }
}

#[derive(Debug, Copy, Clone, Primitive, PartialEq)]
pub(super) enum NAryFlags {
    DontGrowWithContent = 64,
    GrowWithContent = 128,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub(super) struct PhantomDisplay: u8 {
        const PhantomShow = 1;
        const PhantomZeroWidth = 2;
        const PhantomZeroAscent = 4;
        const PhantomZeroDescent = 8;
        const PhantomTransparent = 16;
    }
}

// #[derive(Debug, Copy, Clone)]
// pub(super) enum RadicalDisplay {
//     ShowDegPlaceHldr,
// }

#[derive(Debug, Copy, Clone, Primitive)]
pub(super) enum SubSupAlignment {
    SubSupAlign = 1,
}

#[derive(Debug, Copy, Clone)]
pub(super) enum PhantomKind {
    FullOrCustom,
    HorizontalPhantom,
    VerticalPhantom,
    AscentSmash,
    DescentSmash,
    HorizontalSmash,
    VerticalSmash,
}

#[derive(Debug, Copy, Clone, Primitive)]
pub(super) enum StretchStackPosition {
    CharBelow = 0,
    CharAbove = 1,
    BaseBelow = 2,
    BaseAbove = 3,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(super) struct BoxFlags: u8 {
        const NoBreak = 128;
        // FIXME: literal out of range for `u8`
        // const TransparentForPositioning = 256;
        // const TransparentForSpacing = 512;
    }
}
