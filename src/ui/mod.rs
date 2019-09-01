use std::sync::mpsc::Sender;
use web_view::*;
use crate::message::{SoundMessage, VolumeChange};

pub fn run(
	tx: Sender<SoundMessage>,
	gamelog_path: Option<std::path::PathBuf>,
	soundpack_path: Option<std::path::PathBuf>,
	ignore_path: Option<std::path::PathBuf>,
) {
	let html = format!(r#"
		<!doctype html>
		<html>
			<head>
				<style type="text/css"> {style} </style>
			</head>
			<body>
				<div id="header">
					<button onclick="external.invoke('load_gamelog')">Load gamelog.txt</button>
					<button onclick="external.invoke('load_soundpack')">Load soundpack</button>
					<button onclick="external.invoke('load_ignore_list')">Load ignore.txt</button>
					<button onclick="external.invoke('show_about')">About</button>
				</div>
				<div id="channels"/>
			</body>
		</html>"#,
		style = include_str!("style.css")
	);
	let webview = builder()
		.title("SoundSense-rs")
        .content(Content::Html(html))
        .size(500, 600)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|webview, arg| {
			match arg {
				"load_gamelog" => if let Some(path) = webview.dialog()
					.open_file("Choose gamelog.txt", "")
					.unwrap() {
					tx.send(SoundMessage::ChangeGamelog(path)).unwrap()
				}
				"load_soundpack" => if let Some(path) = webview.dialog()
					.choose_directory("Choose soundpack directory", "")
					.unwrap() {
					tx.send(SoundMessage::ChangeSoundpack(path, UIHandle::new(webview.handle()))).unwrap()
				}
				"load_ignore_list" => if let Some(path) = webview.dialog()
					.open_file("Choose ignore.txt", "")
					.unwrap() {
					tx.send(SoundMessage::ChangeIgnoreList(path)).unwrap()
				}
				"show_about" => {
					webview.dialog().info("SoundSense-rs", "Created by prixt\nThe original SoundSense can be found at:\n\thttp://df.zweistein.cz/soundsense/ \nSource at:\n\thttps://github.com/prixt/soundsense-rs").unwrap()
				}
				other => {
					if let Ok(VolumeChange{channel, volume}) = serde_json::from_str(other) {
						tx.send(SoundMessage::VolumeChange(channel, volume)).unwrap()
					} else {
						unreachable!("Unrecognized argument: {}", other)
					}
				}
			}
			Ok(())
		})
        .build()
        .unwrap();
	
	if let Some(path) = gamelog_path {
		tx.send(SoundMessage::ChangeGamelog(path)).unwrap();
	}
	if let Some(path) = soundpack_path {
		tx.send(SoundMessage::ChangeSoundpack(path, UIHandle::new(webview.handle()))).unwrap();
	}
	if let Some(path) = ignore_path {
		tx.send(SoundMessage::ChangeIgnoreList(path)).unwrap();
	}
	
	webview.run().unwrap();
}

pub struct UIHandle {
	handle: Handle<()>,
	channels: Vec<Box<str>>,
}

impl UIHandle {
	pub fn new(handle: Handle<()>) -> Self {
		Self {
			handle, channels: vec![],
		}
	}
	pub fn add_slider(&mut self, name: String) {
		let name = name.into_boxed_str();
		if !self.channels.contains(&name){
			self.channels.push(name.clone());
			self.handle.dispatch(
				move |webview| {
					let script = format!(r#"
						var slidelabel = document.createElement("LABEL");
						var textnode = document.createTextNode("{channel_name}");
						slidelabel.appendChild(textnode);

						var slider = document.createElement("INPUT");
						slider.setAttribute("type", "range");
						slider.setAttribute("step", "any");
						slider.setAttribute("min", "0");
						slider.setAttribute("max", "100");
						slider.setAttribute("value", "100");
						slider.addEventListener(
							/MSIE|Trident|Edge/.test(window.navigator.userAgent) ? 'change' : 'input',
							function() {{
								external.invoke('{{"channel":"{channel_name}", "volume":'+this.value+'}}');
							}},
							false
						);

						var new_form = document.createElement("FORM");
						new_form.appendChild(slidelabel); new_form.appendChild(slider);

						document.getElementById("channels").appendChild(new_form);
					"#, channel_name=&name);
					webview.eval(&script)
				}
			).unwrap();
		}
	}
	pub fn clear_sliders(&mut self) {
		self.handle.dispatch(
			|webview| {
				webview.eval(r#"
					var channels = document.getElementById("channels");
					while (channels.firstChild) {
						channels.removeChild(channels.firstChild);
					}
				"#)
			}
		).unwrap();
	}
}