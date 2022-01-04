use quantifun::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct State {
	users: HashMap<String, String>,
	sessions: HashMap<String, UserPrincipal>,
}

struct MemAuthService {
	state: Arc<Mutex<State>>,
}

#[async_trait::async_trait]
impl AuthService for MemAuthService {
	async fn register(&self, req: &RegisterRequest) -> RegisterResponse {
		let state = self.state.lock().unwrap();
		state.users.insert(req.email.clone(), req.password);
		RegisterResponse {}
	}

	async fn login(&self, req: &LoginRequest) -> LoginResponse {
		let state = self.state.lock().unwrap();
	}

	async fn test(&self, req: &TestRequest, caller: &UserPrincipal) -> TestResponse {}
}

fn main() {}
