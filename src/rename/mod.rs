mod bibtex_entry;
mod latex_cmd;
mod latex_env;
mod latex_label;

use self::{
    bibtex_entry::{BibtexEntryPrepareRenameProvider, BibtexEntryRenameProvider},
    latex_cmd::{LatexCommandPrepareRenameProvider, LatexCommandRenameProvider},
    latex_env::{LatexEnvironmentPrepareRenameProvider, LatexEnvironmentRenameProvider},
    latex_label::{LatexLabelPrepareRenameProvider, LatexLabelRenameProvider},
};
use crate::{
    feature::{ChoiceProvider, FeatureProvider, FeatureRequest},
    protocol::{Range, RenameParams, TextDocumentPositionParams, WorkspaceEdit},
};
use async_trait::async_trait;

pub struct PrepareRenameProvider {
    provider: ChoiceProvider<TextDocumentPositionParams, Range>,
}

impl PrepareRenameProvider {
    pub fn new() -> Self {
        Self {
            provider: ChoiceProvider::new(vec![
                Box::new(BibtexEntryPrepareRenameProvider),
                Box::new(LatexCommandPrepareRenameProvider),
                Box::new(LatexEnvironmentPrepareRenameProvider),
                Box::new(LatexLabelPrepareRenameProvider),
            ]),
        }
    }
}

impl Default for PrepareRenameProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FeatureProvider for PrepareRenameProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Range>;

    async fn execute<'a>(
        &'a self,
        req: &'a FeatureRequest<TextDocumentPositionParams>,
    ) -> Option<Range> {
        self.provider.execute(req).await
    }
}

pub struct RenameProvider {
    provider: ChoiceProvider<RenameParams, WorkspaceEdit>,
}

impl RenameProvider {
    pub fn new() -> Self {
        Self {
            provider: ChoiceProvider::new(vec![
                Box::new(BibtexEntryRenameProvider),
                Box::new(LatexCommandRenameProvider),
                Box::new(LatexEnvironmentRenameProvider),
                Box::new(LatexLabelRenameProvider),
            ]),
        }
    }
}

impl Default for RenameProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FeatureProvider for RenameProvider {
    type Params = RenameParams;
    type Output = Option<WorkspaceEdit>;

    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<RenameParams>,
    ) -> Option<WorkspaceEdit> {
        self.provider.execute(request).await
    }
}
