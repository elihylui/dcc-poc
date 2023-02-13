use std::fmt::Display;
use std::sync::Mutex;

use actix_web::{
    get, 
    post, 
    error::ResponseError,
    web::{Path, self},
    web::{Json, JsonBody},
    web::Data,
    HttpResponse,
    http::{header::ContentType, StatusCode}, Responder, body::BoxBody, HttpRequest, HttpServer, App
};
use azure_core::error::HttpError;
use azure_messaging_servicebus::service_bus::QueueClient;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Hello{
  id: u32,
  name: String,
}

impl Responder for Hello {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let res_body = serde_json::to_string(&self).unwrap();

        // Create HttpResponse and set Content Type
        HttpResponse::Ok()
           .content_type(ContentType::json())
           .body(res_body)
    }
}

#[derive(Debug, Serialize)]
struct ErrNoId {
  id: u32,
  err: String,
}

// Implement ResponseError for ErrNoId
impl ResponseError for ErrNoId {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
 
    fn error_response(&self) -> HttpResponse<BoxBody> {
       let body = serde_json::to_string(&self).unwrap();
       let res = HttpResponse::new(self.status_code());
       res.set_body(BoxBody::new(body))
    }
 }

 // Implement Display for ErrNoId
impl Display for ErrNoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       write!(f, "{:?}", self)
    }
 }

 struct AppState {
    hello: Mutex<Vec<Hello>>,
 }

#[post("/hello")]
async fn post(req: web::Json<Hello>, data: web::Data<AppState>) -> impl Responder {
   let new_hello = Hello {
       id: req.id,
       name: String::from(&req.name),
   };

   let mut hello = data.hello.lock().unwrap();

   let response = serde_json::to_string(&new_hello).unwrap();

   hello.push(new_hello);
   HttpResponse::Created()
       .content_type(ContentType::json())
       .body(response)
}

#[get("/get/all")]
async fn get_all(data: web::Data<AppState>) -> impl Responder {
   let tickets = data.hello.lock().unwrap();

   let response = serde_json::to_string(&(*tickets)).unwrap();

   HttpResponse::Ok()
       .content_type(ContentType::json())
       .body(response)
}

// Get with the corresponding id
#[get("/get/{id}")]
async fn get(id: web::Path<u32>, data: web::Data<AppState>) -> Result<Hello, ErrNoId> {
   let ticket_id: u32 = *id;
   let tickets = data.hello.lock().unwrap();

   let ticket: Vec<_> = tickets.iter()
                               .filter(|x| x.id == ticket_id)
                               .collect();

   if !ticket.is_empty() {
       Ok(Hello {
           id: ticket[0].id,
           name: String::from(&ticket[0].name)
       })
   } else {
       let response = ErrNoId {
           id: ticket_id,
           err: String::from("id not found")
       };
       Err(response)
   }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
let app_state = web::Data::new(AppState {
                    hello: Mutex::new(vec![
                        Hello {id:1,name:String::from("Jane Doe") },
                        Hello {id:2,name:String::from("Patrick Star")}
                    ])
                });

   HttpServer::new(move || {
       App::new()
           .app_data(app_state.clone())
           .service(post)
           .service(get_all)
           .service(get)
   })
   .bind(("127.0.0.1", 8000))?
   .run()
   .await
}