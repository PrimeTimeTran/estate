// ## AI-Native SDLC playbook
// The AI-native SDLC is a reimagined process that combines the old control objectives with new enforcement. Instead of a linear flow, the process becomes a loop, and AI is embedded at each point. The AI-native SDLC promotes automated handover and triggering of subsequent plays, helping to address the manual and clunky nature of handoff between the phases of the traditional SDLC.
//
// ### 1. Plan
// Requirements gathered by committee, distilled through workshops and sign-offs, written up by hand
// ### 2. Design
// Spec written by analysts, parsed by designers
// ### 3. Test
// Tests and code are handwritten and documentation is written after the main development happens
// ### 4. Deploy
// QA gates at stage boundaries
// ### 5. Maintain
// Humans review every line of code and governance occurs in review cycles, often inconsistently

// ## Reference
// - https://claude.com/blog/the-ai-native-sdlc-playbook

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	use jev_sdk::{Choice, Noul, Question, Score, TypeSafeClient};
	dotenvy::dotenv().ok();
	let key = std::env::var("TYPESAFE_API_KEY")?;

	let client = TypeSafeClient::from_env()?;

	let ticket = "Hi, I've been trying to connect my Stripe account for 3 days \
	              and it keeps failing. I'm losing sales. Please help ASAP.";

	let response = client
		.system_one(
			ticket,
			[
				(
					"department",
					Question::from(Choice::new(
						"Which team should handle this",
						[
							("billing", Some("Payment or subscription issues")),
							("technical", Some("Bugs or integration problems")),
							("sales", Some("Pricing or account questions")),
						],
					)),
				),
				(
					"frustration",
					Question::from(Score::new(
						"How frustrated the customer appears",
						[
							"Calm, just stating facts",
							"Frustrated but civil",
							"Very angry",
						],
					)),
				),
				(
					"is_urgent",
					Question::from(Noul::new("The message conveys urgency or time-sensitivity")),
				),
			],
		)
		.await?;

	let department = response.choice("department").unwrap();

	println!(
		"route to {} (confidence {:.2})",
		department.choice, department.confidence
	);

	println!(
		"frustration: {:.2}",
		response.score("frustration").unwrap().score
	);

	println!("urgent: {:.3}", response.noul("is_urgent").unwrap().noul);

	Ok(())
}
