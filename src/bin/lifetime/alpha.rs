use learn::{
	compiler::*,
	lifetime::{self, *},
	test::*,
	vfs::*,
};

fn main() {
	// lifetime::one();
	// lifetime::two();
	// lifetime::three();
	// lifetime::four();
	lifetime::five();
}

// fn main() {
//     // 1. Diagnostics live in the main scope (they outlive the context!)
//     let mut diagnostics = DiagnosticStore::default();

//     {
//         // 2. The Context is created, does its work, and is then dropped
//         let mut ctx = CompilerContext::new("source code");

//         // Pass a mutable reference to the diagnostics to the context
//         // OR collect them after the stages run
//         run_compiler_pipeline(&mut ctx, &mut diagnostics);
//     }

//     // 3. The context is dead now, but diagnostics are still alive!
//     // We can safely print them to the CLI user.
//     diagnostics.print_report();
// }
