use serde::Deserialize;
use std::cmp::Ordering;

#[derive(Deserialize, Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct CrateDisambiguator(pub u64, pub u64);

#[derive(Deserialize, Debug)]
pub struct CrateId {
	pub name :String,
	pub disambiguator :CrateDisambiguator,
}

#[derive(Deserialize, Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct ItemId<KrateId> {
	pub krate :KrateId,
	pub index :u32,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
	pub file_name :String,
	pub line_start :u32,
	pub line_end :u32,
	pub column_start :u32,
	pub column_end :u32,
}

impl Span {
	/// Obtains file_name.rs:10:32 like format of the span
	pub fn display_str(&self) -> String {
		format!("{}:{}:{}", self.file_name, self.line_start, self.column_start)
	}
}

#[derive(Deserialize, Debug)]
pub struct ExternalCrate {
	pub num :u32,
	pub id :CrateId,
}

#[derive(Deserialize, Debug)]
pub struct Prelude {
	pub crate_id :CrateId,
	pub external_crates :Vec<ExternalCrate>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Def<KrateId> {
	pub kind :String,
	pub name :String,
	pub id :ItemId<KrateId>,
	pub span :Span,
	pub parent :Option<ItemId<KrateId>>,
	pub decl_id :Option<ItemId<KrateId>>,
}


impl<Id :Eq> Ord for Def<Id> {
	fn cmp(&self, other :&Self) -> Ordering {
		(&self.span, &self.name).cmp(&(&other.span, &other.name))
	}
}

impl<Id :Eq> PartialOrd for Def<Id> {
	fn partial_cmp(&self, other :&Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Deserialize, Debug)]
pub struct Ref<KrateId> {
	pub kind :String,
	pub ref_id :ItemId<KrateId>,
	pub span :Span,
}

#[derive(Deserialize, Debug)]
pub struct Compilation {
	pub directory :String,
}

#[derive(Deserialize, Debug)]
pub struct CrateSaveAnalysis {
	pub compilation :Compilation,
	pub prelude :Prelude,
	pub defs :Vec<Def<u32>>,
	pub refs :Vec<Ref<u32>>,
}

#[derive(Deserialize, Debug)]
pub struct CrateSaveAnalysisMetadata {
	pub compilation :Compilation,
	pub prelude :Prelude,
}
