// Copyright 2025 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Labels for conflicted trees.

use std::fmt;
use std::sync::Arc;

use crate::content_hash::ContentHash;
use crate::merge::Merge;

/// Contains a set of optional conflict labels for a merge. The conflict labels
/// are reference-counted to make them more efficient to share.
#[derive(ContentHash, PartialEq, Eq, Clone)]
pub struct ConflictLabels {
    labels: Option<Arc<Merge<String>>>,
}

impl ConflictLabels {
    /// Create a `ConflictLabels` with no labels.
    pub const fn unlabeled() -> Self {
        Self { labels: None }
    }

    /// Create a `ConflictLabels` from an optional `Merge<String>`. If the merge
    /// is resolved, the label will be discarded, since resolved merges cannot
    /// have labels.
    pub fn new(labels: Option<Merge<String>>) -> Self {
        Self {
            labels: labels.filter(|merge| !merge.is_resolved()).map(Arc::new),
        }
    }

    /// Create a `ConflictLabels` from a `Vec<String>`, with an empty vec
    /// representing no labels.
    pub fn from_vec(labels: Vec<String>) -> Self {
        let merge = (!labels.is_empty()).then(|| Merge::from_vec(labels));
        Self::new(merge)
    }

    /// Returns true if there are labels present.
    pub fn is_present(&self) -> bool {
        self.labels.is_some()
    }

    /// Returns the number of labeled sides, or `None` if unlabeled.
    pub fn num_sides(&self) -> Option<usize> {
        self.labels.as_ref().map(|labels| labels.num_sides())
    }

    /// Returns the underlying labels as an `Option<&Merge<String>>`.
    pub fn as_merge(&self) -> Option<&Merge<String>> {
        self.labels.as_ref().map(Arc::as_ref)
    }

    /// Returns the underlying labels as an `Option<Merge<String>>`, cloning if
    /// necessary.
    pub fn into_merge(self) -> Option<Merge<String>> {
        self.labels.map(Arc::unwrap_or_clone)
    }

    /// Returns the conflict labels as a slice. If there are no labels, returns
    /// an empty slice.
    pub fn as_slice(&self) -> &[String] {
        self.as_merge().map_or(&[], |labels| labels.as_slice())
    }

    /// Returns optional labels for each term in a merge. If the merge is
    /// resolved, returns `resolved_label` instead.
    pub fn by_term<'a>(
        &'a self,
        num_sides: usize,
        resolved_label: Option<&'a str>,
    ) -> Merge<Option<&'a str>> {
        if num_sides == 1 {
            assert!(self.labels.is_none());
            Merge::resolved(resolved_label)
        } else {
            self.labels.as_ref().map_or_else(
                || Merge::repeated(None, num_sides),
                |labels| {
                    assert_eq!(num_sides, labels.num_sides());
                    labels.map(|label| Some(label.as_str()))
                },
            )
        }
    }
}

impl From<Option<Merge<String>>> for ConflictLabels {
    fn from(value: Option<Merge<String>>) -> Self {
        Self::new(value)
    }
}

impl From<Option<Merge<&'_ str>>> for ConflictLabels {
    fn from(value: Option<Merge<&str>>) -> Self {
        Self::new(value.map(|labels| labels.map(|&label| label.to_owned())))
    }
}

impl fmt::Debug for ConflictLabels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(labels) = self.as_merge() {
            f.debug_tuple("Labeled").field(&labels.as_slice()).finish()
        } else {
            write!(f, "Unlabeled")
        }
    }
}
