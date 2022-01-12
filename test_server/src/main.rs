use quantifun::auth::*;
use quantifun::core::{Error, HttpPrincipalResolver, Result, UserPrincipal};
use quantifun::{actix_web, async_trait};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Default)]
struct State {
	users: HashMap<String, String>,
	sessions: HashMap<String, UserPrincipal>,
}

struct MemUserPrincipalResolver {
	state: Arc<Mutex<State>>,
}

#[async_trait::async_trait(?Send)]
impl HttpPrincipalResolver<UserPrincipal> for MemUserPrincipalResolver {
	async fn resolve(&self, req: actix_web::HttpRequest) -> Result<UserPrincipal> {
		req.headers()
			.get("Authorization")
			.and_then(|h| h.to_str().ok())
			.and_then(|t| t.strip_prefix("Bearer "))
			.and_then(|t| {
				let state = self.state.lock().unwrap();
				state.sessions.get(t).cloned()
			})
			.ok_or(Error::InvalidCredentials)
	}
}

struct MemAuthService {
	state: Arc<Mutex<State>>,
}

#[async_trait::async_trait]
impl Service for MemAuthService {
	async fn register(&self, req: &RegisterRequest) -> Result<RegisterResponse> {
		let mut state = self.state.lock().unwrap();
		state.users.insert(req.email.clone(), req.password.clone());
		Ok(RegisterResponse {})
	}

	async fn login(&self, req: &LoginRequest) -> Result<LoginResponse> {
		let mut state = self.state.lock().unwrap();
		match state.users.get(&req.email) {
			Some(password) if password == &req.password => (),
			_ => return Err(Error::InvalidCredentials),
		};

		let token = Uuid::new_v4().to_string();

		state
			.sessions
			.insert(token.clone(), UserPrincipal { id: req.email.clone() });

		Ok(LoginResponse { token })
	}

	async fn test(&self, _req: &TestRequest, caller: &UserPrincipal) -> Result<TestResponse> {
		Ok(TestResponse {
			principal_id: caller.id.clone(),
		})
	}

	async fn verify(&self, req: &VerifyRequest) -> Result<VerifyResponse> {
		println!("verifying {}", req.token);
		Ok(VerifyResponse {})
	}
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
	let state = Arc::new(Mutex::new(State::default()));

	actix_web::HttpServer::new(move || {
		actix_web::App::new()
			.app_data(actix_web::web::Data::from(
				Arc::new(MemUserPrincipalResolver { state: state.clone() })
					as Arc<dyn HttpPrincipalResolver<UserPrincipal>>,
			))
			.service(create_scope(Arc::new(MemAuthService { state: state.clone() })))
	})
	.bind(("0.0.0.0", 1337))?
	.run()
	.await
}
