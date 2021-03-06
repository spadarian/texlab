use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::{
        Position, Range, RangeExt, RenameParams, TextDocumentPositionParams, TextEdit,
        WorkspaceEdit,
    },
    syntax::{Span, SyntaxNode},
    workspace::DocumentContent,
};
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexLabelPrepareRenameProvider;

#[async_trait]
impl FeatureProvider for LatexLabelPrepareRenameProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Range>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let pos = req.params.position;
        find_label(&req.current().content, pos).map(Span::range)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexLabelRenameProvider;

#[async_trait]
impl FeatureProvider for LatexLabelRenameProvider {
    type Params = RenameParams;
    type Output = Option<WorkspaceEdit>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let pos = req.params.text_document_position.position;
        let name = find_label(&req.current().content, pos)?;
        let mut changes = HashMap::new();
        for doc in req.related() {
            if let DocumentContent::Latex(table) = &doc.content {
                let edits = table
                    .labels
                    .iter()
                    .flat_map(|label| label.names(&table))
                    .filter(|label| label.text() == name.text)
                    .map(|label| TextEdit::new(label.range(), req.params.new_name.clone()))
                    .collect();
                changes.insert(doc.uri.clone().into(), edits);
            }
        }
        Some(WorkspaceEdit::new(changes))
    }
}

fn find_label(content: &DocumentContent, pos: Position) -> Option<&Span> {
    if let DocumentContent::Latex(table) = content {
        table
            .labels
            .iter()
            .flat_map(|label| label.names(&table))
            .find(|label| label.range().contains(pos))
            .map(|label| &label.span)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use indoc::indoc;

    #[tokio::test]
    async fn label() {
        let actual_edit = FeatureTester::new()
            .file(
                "foo.tex",
                indoc!(
                    r#"
                        \label{foo}
                        \include{bar}
                    "#
                ),
            )
            .file("bar.tex", r#"\ref{foo}"#)
            .file("baz.tex", r#"\ref{foo}"#)
            .main("foo.tex")
            .position(0, 7)
            .new_name("bar")
            .test_rename(LatexLabelRenameProvider)
            .await
            .unwrap();

        let mut expected_changes = HashMap::new();
        expected_changes.insert(
            FeatureTester::uri("foo.tex").into(),
            vec![TextEdit::new(Range::new_simple(0, 7, 0, 10), "bar".into())],
        );
        expected_changes.insert(
            FeatureTester::uri("bar.tex").into(),
            vec![TextEdit::new(Range::new_simple(0, 5, 0, 8), "bar".into())],
        );

        assert_eq!(actual_edit, WorkspaceEdit::new(expected_changes));
    }

    #[tokio::test]
    async fn command_args() {
        let actual_edit = FeatureTester::new()
            .file("main.tex", r#"\foo{bar}"#)
            .main("main.tex")
            .position(0, 5)
            .new_name("baz")
            .test_rename(LatexLabelRenameProvider)
            .await;

        assert_eq!(actual_edit, None);
    }

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_edit = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .new_name("")
            .test_rename(LatexLabelRenameProvider)
            .await;

        assert_eq!(actual_edit, None);
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_edit = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .new_name("")
            .test_rename(LatexLabelRenameProvider)
            .await;

        assert_eq!(actual_edit, None);
    }
}
