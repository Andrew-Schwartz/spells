// use futures::StreamExt;
// use itertools::Itertools;
// use reqwest::{Client, Response};
// use select::document::Document;
// use select::node::Node;
// use select::predicate::{Attr, Text};
// use serde::Serialize;
// use tokio::fs::File;
// use tokio::io::AsyncReadExt;
// use tokio::time::Duration;
//
// #[allow(dead_code)]
// pub async fn fetch() -> anyhow::Result<()> {
//     // let response = get("https://www.dnd-spells.com/spells").await?;
//     // let mut file = File::create("data/spells.html").await?;
//     // file.write_all(response.text().await?.as_bytes()).await?;
//     let mut file = File::open("data/spells.html").await?;
//     let mut str = String::new();
//     file.read_to_string(&mut str).await?;
//     let doc = Document::from(str.as_str());
//     let listings: Vec<_> = doc.select(Attr("type", "checkbox"))
//         .filter(|n| n.attr("name").map_or(false, |name| name == "spells[]"))
//         .map(|n| n.parent().unwrap().parent().unwrap())
//         .map(SpellListing::from)
//         // "Trap the soul shouldn’t appear on the spell list"
//         .filter(|s| s.name != "Trap the Soul")
//         .collect();
//     println!("listings.len() = {:?}", listings.len());
//     let client = Client::new();
//     let docs: Vec<_> = futures::stream::iter(&listings)
//         // .skip(409).take(1)
//         .map(|s| s.url)
//         .inspect(|url| println!("url = {:?}", url))
//         .then(|url| client.get(url).send())
//         .map(Result::unwrap)
//         .then(Response::text)
//         .map(Result::unwrap)
//         .map(|text| Document::from(text.as_str()))
//         .then(|doc| async move {
//             tokio::time::sleep(Duration::from_millis(500)).await;
//             doc
//         })
//         .collect()
//         .await;
//     println!("docs.len() = {:?}", docs.len());
//     let mut spells = Vec::new();
//     for (i, spell) in listings.into_iter().enumerate() {
//         print!("{}: ", i);
//     // {
//     //     let i = 0;
//     //     let spell = listings.remove(409);
//         let SpellListing {
//             name,
//             url,
//             level,
//             school,
//             casting_time,
//             ritual,
//             conc,
//             classes,
//             source
//         } = spell;
//         println!("url = {}", url);
//         let doc = &docs[i];
//         let skip = if source == "Players Handbook" { 2 } else { 3 };
//         let mut elems = doc.select(Attr("class", "col-md-12"))
//             .flat_map(|n| n.children())
//             .filter(|n| !n.is(Text))
//             // skip until we get to the name (h1)
//             .skip_while(|n| n.name().map_or(true, |n| n != "h1"))
//             // skip name and school
//             .skip(skip);
//         let mut spell = Spell {
//             name,
//             level,
//             casting_time,
//             range: "",
//             duration: "",
//             components: "",
//             school,
//             ritual,
//             conc,
//             description: "".into(),
//             higher_levels: None,
//             classes,
//             source,
//             page: 0,
//         };
//         let info = elems.next().unwrap();
//         for chunk in &info.children().chunks(2) {
//             // last one is just a Text
//             if let Some((name, has_child)) = chunk.collect_tuple::<(Node, Node)>() {
//                 let child_text = || has_child.first_child()
//                     .unwrap()
//                     .as_text()
//                     .unwrap()
//                     .trim();
//                 #[allow(clippy::match_same_arms)]
//                 match name.as_text().map(str::trim) {
//                     Some("Level:") => spell.level = match child_text() {
//                         "Cantrip" => 0,
//                         num => num.parse().unwrap(),
//                     },
//                     // this is already filled in from the listing
//                     Some("Casting time:") => {}
//                     // Some("Casting time:") => spell.casting_time = child_text(),
//                     Some("Range:") => spell.range = child_text(),
//                     Some("Components:") => spell.components = child_text(),
//                     Some("Duration:") => spell.duration = child_text(),
//                     _ => {}
//                 }
//             }
//         }
//         // elems.for_each(|n| println!("n = {:?}", n));
//         let mut elems = elems.skip(2);
//         let mut desc = elems.next().unwrap()
//             .children()
//             .map(|n| n.as_text()
//                 .or_else(|| n.first_child().and_then(|n| n.as_text()))
//                 .unwrap_or(""))
//             .join("");
//         let mut elems = elems.peekable();
//
//         // can have more lines of description in `div`s
//         let mut dived = false;
//         while elems.peek()
//             .and_then(Node::name)
//             .map_or(false, |name| name == "div")
//         {
//             dived = true;
//
//             elems.next().unwrap()
//                 .children()
//                 .map(|n| n.as_text()
//                     .or_else(|| n.first_child().and_then(|n| n.as_text()))
//                     .unwrap_or("")
//                 )
//                 .for_each(|str| {
//                     desc.push('\n');
//                     desc.push_str(str);
//                 });
//         }
//         spell.description = desc.trim().to_string();
//         // if there is stuff in div it seems like there's also another <p>
//         if dived {
//             elems.next().unwrap();
//         }
//
//         while elems.peek()
//             .and_then(Node::name)
//             .map_or(false, |name| name == "p")
//         {
//             elems.next().unwrap();
//         }
//         let at_higher_levels = elems.next().unwrap().name().map_or(false, |n| n == "h4");
//         if at_higher_levels {
//             let higher = elems.next().unwrap();
//             spell.higher_levels = Some(
//                 higher.children()
//                     .map(|n| n.as_text()
//                         .or_else(|| n.first_child().and_then(|n| n.as_text()))
//                         .unwrap_or("")
//                     )
//                     .join("")
//                     .trim()
//                     .to_string()
//             );
//             // there's an extra node in this case
//             elems.next().unwrap();
//         }
//
//         // ex: "Page: 223  Players Handbook"
//         let source_info = elems.next()
//             .unwrap()
//             .first_child()
//             .unwrap()
//             .as_text()
//             .unwrap()
//             .trim();
//         // println!("source_info = {:?}", source_info);
//         let start = source_info.find(':').unwrap() + 2;
//         let end = source_info[start..].find(' ').unwrap();
//         spell.page = source_info[start..start + end].parse().unwrap();
//
//         spells.push(spell);
//     }
//
//     let file = std::fs::File::create("data/json")?;
//     serde_json::to_writer(file, &spells).unwrap();
//
//     Ok(())
// }
//
// #[derive(Debug, Serialize)]
// struct Spell<'doc> {
//     name: &'doc str,
//     level: u32,
//     casting_time: &'doc str,
//     range: &'doc str,
//     duration: &'doc str,
//     components: &'doc str,
//     school: &'doc str,
//     ritual: bool,
//     conc: bool,
//     description: String,
//     higher_levels: Option<String>,
//     classes: Vec<&'doc str>,
//     source: &'doc str,
//     page: u32,
// }
//
// #[derive(Debug)]
// struct SpellListing<'doc> {
//     name: &'doc str,
//     url: &'doc str,
//     level: u32,
//     school: &'doc str,
//     casting_time: &'doc str,
//     ritual: bool,
//     conc: bool,
//     classes: Vec<&'doc str>,
//     source: &'doc str,
// }
//
// impl<'doc> From<Node<'doc>> for SpellListing<'doc> {
//     fn from(node: Node<'doc>) -> Self {
//         let mut this = Self {
//             name: "",
//             url: "",
//             level: 0,
//             school: "",
//             casting_time: "",
//             ritual: false,
//             conc: false,
//             classes: vec![],
//             source: "",
//         };
//         let mut fields = node.children()
//             .filter(|n| !n.is(Text))
//             .skip(1);
//         let name_url = fields.next().unwrap();
//         name_url.children()
//             .next().unwrap()
//             .attrs()
//             .for_each(|(name, str)| match name {
//                 "title" => this.name = {
//                     // let idx = str.find(", a level").unwrap();
//                     let idx = str.find(',').unwrap();
//                     &str[..idx]
//                 },
//                 "href" => this.url = str,
//                 _ => {}
//             });
//         let mut next_text = || {
//             fields.next()
//                 .unwrap()
//                 .first_child()
//                 .unwrap()
//                 .as_text()
//                 .unwrap()
//                 .trim()
//         };
//         let yes_no = |str| match str {
//             "Yes" => true,
//             "No" => false,
//             bad => unreachable!("bad = {}", bad),
//         };
//         this.level = next_text().parse().unwrap();
//         this.school = next_text();
//         this.casting_time = next_text();
//         this.ritual = yes_no(next_text());
//         this.conc = yes_no(next_text());
//         this.classes = next_text().split_whitespace().collect();
//         this.source = next_text();
//         this
//     }
// }