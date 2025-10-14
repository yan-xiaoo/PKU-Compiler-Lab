use codespan_reporting::diagnostic::Diagnostic;

use crate::function_ast::Span;

/// 错误等级：错误/警告
#[allow(dead_code)]
pub enum ProblemLevel {
    Error,
    Warning
}

/// 标签等级：首要和次要（红色/蓝色）
pub enum LabelLevel {
    Primary,
    Secondary
}

/// 存储错误 tag（描述）的结构体
pub struct Label {
    pub level: LabelLevel,
    pub message: String,
    pub span: Span
}

/// 一个存储单个编译错误信息的结构体
#[allow(dead_code)]
pub struct ProblemInfo {
    pub code: Option<String>,
    pub message: String, 
    pub level: ProblemLevel,
    pub labels: Vec<Label>,
    pub notes: Vec<String>
}

#[allow(dead_code)]
impl Label {
    pub fn primary<T>(message: T, span: Span) -> Self
        where T: ToString
    {
        Self {
            level: LabelLevel::Primary,
            message: message.to_string(),
            span
        }
    }

    pub fn secondary<T>(message: T, span: Span) -> Self
        where T: ToString
    {
        Self {
            level: LabelLevel::Secondary,
            message: message.to_string(),
            span
        }
    }
}


impl ProblemInfo {
    #[allow(dead_code)]
    pub fn error<T>(message: T, labels: Vec<Label>, notes: Option<Vec<String>>) -> Self
        where T: ToString
    {
        let notes = notes.unwrap_or_default();
        Self {
            code: None,
            message: message.to_string(),
            level: ProblemLevel::Error,
            labels,
            notes
        }
    }

    pub fn warning<T>(message: T, labels: Vec<Label>, notes: Option<Vec<String>>) -> Self
        where T: ToString
    {
        let notes = notes.unwrap_or_default();
        Self {
            code: None,
            message: message.to_string(),
            level: ProblemLevel::Warning,
            labels,
            notes
        }
    }

    pub fn generate(&self, file_id: usize) -> Diagnostic<usize> {
        let diagnostic = match self.level {
            ProblemLevel::Error => {
                Diagnostic::error()
            },
            ProblemLevel::Warning => {
                Diagnostic::warning()
            }
        };
        let diagnostic = diagnostic.with_message(&self.message);
        let diagnostic = if let Some(code) = &self.code {
            diagnostic.with_code(code)
        } else {
            diagnostic
        };
        let mut labels = Vec::new();
        for label in &self.labels {
            let single =  match label.level {
                LabelLevel::Primary => {
                    codespan_reporting::diagnostic::Label::primary(file_id, label.span.start..label.span.end).with_message(&label.message)
                },
                LabelLevel::Secondary => {
                    codespan_reporting::diagnostic::Label::secondary(file_id, label.span.start..label.span.end).with_message(&label.message)
                }
            };
            labels.push(single);
        }
        diagnostic.with_labels(labels).with_notes(self.notes.iter().map(|one| unindent::unindent(one)).collect())
    }
}