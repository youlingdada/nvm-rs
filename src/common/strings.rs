use regex::Regex;
use std::env;

// s 是否包含chars 中任何一个字符
pub fn contains_any(s: &str, chars: &str) -> bool {
    chars.chars().any(|c| s.contains(c))
}

// 将字符串的环境变量替换为实际值
pub fn replace_env_vars(input: &str) -> String {
    // 正则表达式匹配 ${VAR_NAME} 或 $VAR_NAME 格式的环境变量占位符
    let re = Regex::new(r"\$\{([^}]+)\}|\$([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();

    // 使用闭包进行替换操作
    re.replace_all(input, |caps: &regex::Captures| {
        if let Some(var_name) = caps.get(1) {
            // ${VAR_NAME} 格式
            env::var(var_name.as_str()).unwrap_or_else(|_| "".to_string())
        } else if let Some(var_name) = caps.get(2) {
            // $VAR_NAME 格式
            env::var(var_name.as_str()).unwrap_or_else(|_| "".to_string())
        } else {
            String::new()
        }
    })
    .to_string()
}
