use quantifun::*;
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
	async fn resolve(&self, req: actix_web::HttpRequest) -> Result<UserPrincipal, actix_web::HttpResponse> {
		req.headers()
			.get("Authorization")
			.and_then(|h| h.to_str().ok())
			.and_then(|t| t.strip_prefix("Bearer "))
			.and_then(|t| {
				let state = self.state.lock().unwrap();
				state.sessions.get(t).cloned()
			})
			.ok_or_else(|| actix_web::HttpResponse::Unauthorized().finish())
	}
}

struct MemAuthService {
	state: Arc<Mutex<State>>,
}

#[async_trait::async_trait]
impl AuthService for MemAuthService {
	async fn register(&self, req: &RegisterRequest) -> Result<RegisterResponse, AuthError> {
		let mut state = self.state.lock().unwrap();
		state.users.insert(req.email.clone(), req.password.clone());
		Ok(RegisterResponse {})
	}

	async fn login(&self, req: &LoginRequest) -> Result<LoginResponse, AuthError> {
		let mut state = self.state.lock().unwrap();
		match state.users.get(&req.email) {
			Some(password) if password == &req.password => (),
			_ => return Err(AuthError::InvalidCredentials),
		};

		let token = Uuid::new_v4().to_string();

		state
			.sessions
			.insert(token.clone(), UserPrincipal { id: req.email.clone() });

		Ok(LoginResponse { token })
	}

	async fn test(&self, _req: &TestRequest, caller: &UserPrincipal) -> Result<TestResponse, AuthError> {
		Ok(TestResponse {
			principal_id: caller.id.clone(),
		})
	}
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let state = Arc::new(Mutex::new(State::default()));

	actix_web::HttpServer::new(move || {
		actix_web::App::new()
			.app_data(actix_web::web::Data::from(
				Arc::new(MemUserPrincipalResolver { state: state.clone() })
					as Arc<dyn HttpPrincipalResolver<UserPrincipal>>,
			))
			.service(auth_http_scope(Arc::new(MemAuthService { state: state.clone() })))
	})
	.bind(("0.0.0.0", 1337))?
	.run()
	.await
}
