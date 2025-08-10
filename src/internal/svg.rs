
pub fn split_line(line:&str)->Vec<String>{
    let splitter_char = ",".chars().next().unwrap();
    let quotes_char = "\"".chars().next().unwrap();
    let escape_char = "\\".chars().next().unwrap();
    let skip_char = "\r\t";
    let mut temp = String::new();
    let mut total=Vec::new();
    let mut in_quotes = false;
    let mut escaped = false;
    for i in line.chars(){
        if i == quotes_char{
            if !escaped{
            in_quotes = !in_quotes;
            continue;}else{
                escaped =false;
            }
        }
        if i == escape_char{
            if !escaped{escaped = true;continue;}
            else
                {escaped =false;}
        }
        if i == splitter_char && !in_quotes{
            total.push(temp.trim().to_string());
            temp = "".to_string();
            continue;
        }
        if skip_char.contains(i) && !in_quotes{
            continue;
        }

        temp.push(i);
    }
    total.push(temp.trim().to_string());


    total
}

pub fn parse_translations(
    csv_data: &str,
    desired_language: &str,
    fallback_language: &str
) -> Result<(String, std::collections::HashMap<String, String>), &'static str> {
    let split:Vec<&str> = csv_data.split("\n").collect();
    let language_header = split_line(split[0]);
    let language_idx;
    let used_language;
    if let Some(language_idx_first) = language_header.iter().position(|x| x == desired_language){
        language_idx = language_idx_first;
        used_language = desired_language;
    }else if let Some(language_idx_fallback) = language_header.iter().position(|x|x == fallback_language){
            language_idx = language_idx_fallback;
            used_language = fallback_language;
        }else{
            return Err("Unable to find desired nor fallback languages");
        }
    
    let mut stuff = std::collections::HashMap::new();
    for i in &split[1..]{
        let extracted = split_line(i);
        stuff.insert(extracted[0].clone(), extracted[language_idx].clone());
    }

    Ok((used_language.to_string(), stuff))

}