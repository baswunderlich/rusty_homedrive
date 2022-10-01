use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use rocket::Data;
use rocket::data::ToByteUnit;
use rocket::http::Status;
use rocket::fs::{FileServer};
use rocket::fs::{relative};
use rocket::response::status::{self};
use rocket::tokio::io::AsyncReadExt;

#[macro_use] extern crate rocket;

#[post("/upload?<path>&<name>", data = "<data>")]
async fn upload(name: String, path: String, data: Data<'_>) -> status::Custom<String> {
    match File::create(format!("storage/{}/{}", path, name)){
        Ok(mut file) => {
            let mut my_data = data.open(1.gigabytes());
            let mut buffer: Vec<u8> = Vec::new();
            let read_data_size = my_data.read_to_end(&mut buffer).await.expect("Data couldnt be retrieved");
            file.write_all(&buffer).expect("Data couldnt be written to file");
        },
        Err(_) => return status::Custom(Status::BadRequest, String::from("Writing failed"))
    }
    return status::Custom(Status::Ok, String::from("Successfull upload"))
}


#[post("/create_dir?<path>&<name>")]
async fn create_dir(name: String, path: String) -> status::Custom<String> {
    match fs::create_dir(format!("storage/{}/{}", path, name)){
        Ok(_) => return status::Custom(Status::Ok, String::from("Directory created successfully")),
        Err(_) => return status::Custom(Status::BadRequest, String::from("Directory couldnt be created"))
    }
}


#[post("/delete?<path>")]
fn delete(mut path: String) -> status::Custom<String> {
    path = format!("storage/{}", path);
    match File::open(path.clone()){
        Ok(file) => {
            let md = file.metadata().expect("Error reading the metadata");
            if md.is_dir(){
                fs::remove_dir_all(path.clone()).expect("Error deleting the directory");
            }
            if md.is_file(){
                fs::remove_file(path).expect("Error deleting the file");
            }
            return status::Custom(Status::Ok, String::from("Successfully removed"))
        },
        Err(_) => return status::Custom(Status::BadRequest, String::from("No such file or directory found"))
    }
}

#[get("/list/<path>")]
fn list(mut path: String) -> status::Custom<String>{
    if path == "."{
        path = String::from("storage/");
        println!("Storage set");
    }else{
        path = format!("storage/{}", path);
    }
    let entries = fs::read_dir(path).expect("No such directory found");
    let mut result = String::new();
    for e in entries{
        if e.unwrap().metadata().unwrap().is_dir(){
            result = format!("{}/d{:?}", result, e.unwrap().file_name());
        }
        if e.unwrap().metadata().unwrap().is_file(){
            result = format!("{}/f{:?}", result, e.unwrap().file_name());
        }
    }
    return status::Custom(Status::Ok, result)
}

#[get("/download/<path>")]
fn download(mut path: String) -> status::Custom<Vec<u8>> {
    path = format!("storage/{}",path);
    let mut f = File::open(&path).expect("no file found");
    let metadata = fs::metadata(&path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    status::Custom(Status::Ok, buffer)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
    .mount("/", FileServer::from(relative!("www")))
    .mount("/homedrive", routes![upload, delete, list, download, create_dir])
}