use std::marker::PhantomData;
trait Platform {
	fn name();
}

trait Compute {
	fn state();
	fn is_up();
}

trait Host<C> {
	fn services();
	fn network();
	fn runtime();
}

trait Work<C> {
	fn run() {}
}

trait Act<C> {
	fn run();
	fn start();
	fn up();
	fn down();
}

// Attempt at recursion for Context
trait Ctx<C> {
	type Host: Host<C>;
	type Work: Work<C>;
	type Act: Act<C>;
	type Context: State<C>;
	fn create();
	fn read();
	fn update();
	fn delete();
}

// State is "f of x"
// Function of context
// -> f(context)
trait State<C> {
	type Host: Host<C>;
	type Work: Work<C>;
	type Act: Act<C>;
}

struct Stateful;
struct Stateless;
struct Wasm;
struct Native;

impl<C> Host<C> for Native {
	fn services() {}
	fn network() {}
	fn runtime() {}
}
struct Spec<P> {
	name: String,
	data: PhantomData<P>,
}

enum LifeCycle {
	Start,
	Running,
}
struct Environment<O> {
	os: PhantomData<O>,
}

struct OS<C> {
	name: Environment<C>,
}

struct Context<T> {
	data: PhantomData<T>,
}
impl<T> Context<T> {
	fn inherent_struct_method() {
		println!("Always available")
	}
	fn inherent_instance_method(&self) {
		println!("Always available")
	}
}
impl<H> Context<H>
where
	H: Host<H>,
{
	fn execute() {
		println!("Need a host")
	}
}
impl<W> Context<W>
where
	W: Work<W>,
{
	fn run() {}
}

impl<T> Context<T>
where
	T: Host<T> + Act<T>,
{
	fn os_methods() {}
	fn infrastructure_apis() {}
}
impl<S> Context<S>
where
	S: State<S>,
{
	fn contexts_stateful_methods() {}
	fn contexts_stateless_methods() {}
	fn methods_which_observe_context_state() {}
	fn context_provided_state() {}
}
impl<T> Context<T>
where
	T: State<T>,
{
	fn type_state() {}
	fn type_trait_state() {}
	fn struct_trait_type_state() {}
}

impl OS<Native> {
	fn init() {}
}
impl OS<Wasm> {
	fn init() {}
}

impl<T> Context<T>
where
	T: State<T>,
{
	fn stateful() {}
}
impl<J> Context<J>
// What is the state/context of my background jobs?
// What methods are available?
where
	J: State<J>,
{
	fn create_jobs() {}
	fn read_jobs() {}
	fn update_jobs() {}
	fn delete_jobs() {}
}

impl<P> Act<P> for Context<P> {
	// What is the "state" of my processes
	fn run() {}
	fn start() {}
	fn up() {}
	fn down() {}
}

impl Context<Stateful> {
	fn stateless(&self) {
		// #Works
		self.inherent_instance_method();
		Self::inherent_struct_method();

		// Need additional types.
		// These cause errors.
		// So have to pass them in via generic type params.
		// Or define where impls for the Context
		// self::Context::create_jobs();
		// self::Context::read_jobs();
		// self::Context::update_jobs();
		// self::Context::delete_jobs();
		// self::Context::execute();
	}
}
impl Context<Stateful> {
	fn stateful<J>(&self) {
		// #Works
		self.inherent_instance_method();
		Self::inherent_struct_method();

		// Need additional types.
		// But how to share the methods robustly is the real question
		// When the context is composed of dynamic type/struct/types
		// self::Context::<J>::create_jobs();
		// self::Context::read_jobs();
		// self::Context::update_jobs();
		// self::Context::delete_jobs();
		// self::Context::execute();
	}
}

impl<P> Context<P>
// Context is a f of e, environment
where
	P: Platform,
{
	fn is_platform(&self) {
		Self::inherent_struct_method();
		self.inherent_instance_method()
	}
}
impl<S> Context<S>
// My "End State" is a function of the context in which I'm executing
// it's own 'statefulness' and it's internally resolved 'context' from higher up & lower down resolvers
// Platform, OS, Host, etc.
where
	S: State<S>,
{
	fn is_stateful() {}
}
impl<T> Context<T>
// The state of a type.
where
	T: State<T>,
{
	fn is_typeful() {}
}

impl<T> State<T> for Context<T> {
	type Host = Native;
	type Work = Worker;
	type Act = Runner;
}

struct App<C> {
	ctx: PhantomData<C>,
}

struct Worker;
struct Runner;

impl<C> Work<C> for Worker {
	fn run() {}
}
impl<C> Act<C> for Runner {
	fn run() {}
	fn start() {}
	fn up() {}
	fn down() {}
}

impl<S, T> Ctx<T> for App<S> {
	type Host = Native;
	type Work = Worker;
	type Act = Runner;
	type Context = Context<T>;
	fn create() {}
	fn read() {}
	fn update() {}
	fn delete() {}
}
