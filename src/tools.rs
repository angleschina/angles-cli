use std::process::Command;
use std::fs;
use std::path::Path;

/// Run the doctor diagnostic.
pub fn doctor() {
    println!();
    println!("  α  Angles Code CLI — 诊断报告");
    println!();
    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Binary
    println!("  angles 二进制: 已安装");
    println!("     架构: {} / {}", std::env::consts::ARCH, std::env::consts::OS);

    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Binary
    println!("  angles 二进制: 已安装");

    // Config check
    let cfg = crate::config::load_or_default();
    let cfg_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".angles")
        .join("config.json");
    if cfg_path.exists() {
        println!("  配置文件: {}", cfg_path.display());
        println!("     Provider: {}", cfg.provider);
        println!("     Model: {}", cfg.model);
    } else {
        println!("  配置文件: 未找到 (运行 `angles gateway` 创建)");
    }

    // API Key check
    let key_set = !cfg.api_key.is_empty() || std::env::var("ANGLES_API_KEY").is_ok();
    if key_set {
        println!("  API Key: 已配置");
    } else {
        println!("  API Key: 未配置 (运行 `angles gateway` 设置)");
    }

    // API connectivity test
    if !cfg.base_url.is_empty() {
        let test_url = if cfg.base_url.ends_with("/v1") {
            format!("{}/models", cfg.base_url)
        } else {
            format!("{}/v1/models", cfg.base_url.trim_end_matches('/'))
        };
        let key = if !cfg.api_key.is_empty() { &cfg.api_key }
            else { &std::env::var("ANGLES_API_KEY").unwrap_or_default() };
        match Command::new("curl").args([
            "-s", "-o", "/dev/null", "-w", "%{http_code}",
            "--connect-timeout", "10",
            "-H", &format!("Authorization: Bearer {}", key),
            &test_url,
        ]).output() {
            Ok(o) => {
                let code = String::from_utf8_lossy(&o.stdout).trim().to_string();
                match code.as_str() {
                    "200" => println!("  API 连通: {} → 200 OK", cfg.base_url),
                    "401" => println!("  API 连通: {} → 401 (API Key 无效)", cfg.base_url),
                    "404" => println!("  API 连通: {} → 404 (端点不存在)", cfg.base_url),
                    c if c.starts_with("2") => println!("  API 连通: {} → {}", cfg.base_url, c),
                    c => println!("  API 连通: {} → HTTP {}", cfg.base_url, c),
                }
            }
            _ => println!("  API 连通: 无法连接 {}", cfg.base_url),
        }
    }

    // Git
    match Command::new("git").arg("--version").output() {
        Ok(o) if o.status.success() => {
            let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
            println!("  Git: {}", ver);
        }
        _ => println!("  Git: 未安装"),
    }

    // ripgrep
    match Command::new("rg").arg("--version").output() {
        Ok(o) if o.status.success() => {
            let ver = String::from_utf8_lossy(&o.stdout).lines().next().unwrap_or("").to_string();
            println!("  ripgrep: {}", ver);
        }
        _ => println!("  ripgrep: 未安装 (angles-grep 将使用 grep 替代)"),
    }

    println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
}

// ─── angles-* tool implementations ───

pub fn angles_createfile(path: &str, content: &str) -> Result<String, String> {
    if Path::new(path).exists() {
        return Err(format!("文件已存在: {}", path));
    }
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    fs::write(path, content).map_err(|e| format!("写入失败: {}", e))?;
    Ok(format!("已创建: {}", path))
}

pub fn angles_writefile(path: &str, content: &str) -> Result<String, String> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    fs::write(path, content).map_err(|e| format!("写入失败: {}", e))?;
    Ok(format!("已写入: {}", path))
}

pub fn angles_appendfile(path: &str, content: &str) -> Result<String, String> {
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("打开失败: {}", e))?;
    f.write_all(content.as_bytes()).map_err(|e| format!("追加失败: {}", e))?;
    Ok(format!("已追加到: {}", path))
}

pub fn angles_readfile(path: &str, start: Option<usize>, end: Option<usize>) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("读取失败: {}", e))?;
    let lines: Vec<&str> = content.lines().collect();
    let s = start.unwrap_or(1).saturating_sub(1);
    let e = end.unwrap_or(lines.len()).min(lines.len());
    if s >= lines.len() {
        return Ok(String::new());
    }
    Ok(lines[s..e].iter().enumerate()
        .map(|(i, l)| format!("{:>4} | {}", s + i + 1, l))
        .collect::<Vec<_>>()
        .join("\n"))
}

pub fn angles_replace(path: &str, old: &str, new: &str) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("读取失败: {}", e))?;
    match content.find(old) {
        Some(pos) => {
            let replaced = format!("{}{}{}", &content[..pos], new, &content[pos + old.len()..]);
            fs::write(path, replaced).map_err(|e| format!("写入失败: {}", e))?;
            Ok(format!("已替换 (1处): {}", path))
        }
        None => Err(format!("未找到匹配文本: {}", old)),
    }
}

pub fn angles_replaceall(path: &str, old: &str, new: &str) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("读取失败: {}", e))?;
    let count = content.matches(old).count();
    if count == 0 {
        return Err(format!("未找到匹配文本: {}", old));
    }
    let replaced = content.replace(old, new);
    fs::write(path, replaced).map_err(|e| format!("写入失败: {}", e))?;
    Ok(format!("已替换 ({}处): {}", count, path))
}

pub fn angles_deletefile(path: &str) -> Result<String, String> {
    fs::remove_file(path).map_err(|e| format!("删除失败: {}", e))?;
    Ok(format!("已删除: {}", path))
}

pub fn angles_mkdir(dir: &str) -> Result<String, String> {
    fs::create_dir_all(dir).map_err(|e| format!("创建失败: {}", e))?;
    Ok(format!("已创建目录: {}", dir))
}

pub fn angles_movedir(src: &str, dst: &str) -> Result<String, String> {
    fs::rename(src, dst).map_err(|e| format!("移动失败: {}", e))?;
    Ok(format!("已移动: {} → {}", src, dst))
}

pub fn angles_copyfile(src: &str, dst: &str) -> Result<String, String> {
    fs::copy(src, dst).map_err(|e| format!("复制失败: {}", e))?;
    Ok(format!("已复制: {} → {}", src, dst))
}

pub fn angles_run(cmd: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| format!("执行失败: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let mut result = stdout;
    if !stderr.is_empty() {
        if !result.is_empty() { result.push('\n'); }
        result.push_str(&stderr);
    }
    Ok(result)
}

pub fn angles_searchfile(pattern: &str, directory: &str) -> Result<String, String> {
    let dir = if directory.is_empty() { "." } else { directory };
    let output = Command::new("find")
        .args([dir, "-name", pattern, "-type", "f"])
        .output()
        .map_err(|e| format!("搜索失败: {}", e))?;
    let result = String::from_utf8_lossy(&output.stdout).to_string();
    if result.trim().is_empty() {
        Ok("未找到匹配文件".into())
    } else {
        Ok(result)
    }
}

pub fn angles_grep(pattern: &str, directory: &str) -> Result<String, String> {
    let dir = if directory.is_empty() { "." } else { directory };
    // Try rg first, fall back to grep
    let output = if which::which("rg").is_ok() {
        Command::new("rg").args(["-n", "--no-heading", pattern, dir]).output()
    } else {
        Command::new("grep").args(["-rn", pattern, dir]).output()
    }.map_err(|e| format!("搜索失败: {}", e))?;

    let result = String::from_utf8_lossy(&output.stdout).to_string();
    if result.trim().is_empty() {
        Ok("未找到匹配内容".into())
    } else {
        Ok(result)
    }
}

pub fn angles_websearch(query: &str, engine_url: &str) -> Result<String, String> {
    // For now, return the search URL for the user to open
    // Full scraping would require browser automation or search API
    Ok(format!("搜索链接: {}\n（完整搜索功能需配合浏览器使用）", engine_url))
}

// ─── Additional tools (补齐到 30+) ───

pub fn angles_insertline(path: &str, line_num: usize, content: &str) -> Result<String, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("读取失败: {}", e))?;
    let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    let idx = line_num.saturating_sub(1).min(lines.len());
    lines.insert(idx, content.to_string());
    fs::write(path, lines.join("\n")).map_err(|e| format!("写入失败: {}", e))?;
    Ok(format!("已在第 {} 行前插入: {}", line_num, path))
}

pub fn angles_deleteline(path: &str, line_num: usize) -> Result<String, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("读取失败: {}", e))?;
    let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    let idx = line_num.saturating_sub(1);
    if idx >= lines.len() {
        return Err(format!("行号超出范围: {} (共 {} 行)", line_num, lines.len()));
    }
    lines.remove(idx);
    fs::write(path, lines.join("\n")).map_err(|e| format!("写入失败: {}", e))?;
    Ok(format!("已删除第 {} 行: {}", line_num, path))
}

pub fn angles_head(path: &str, n: usize) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("读取失败: {}", e))?;
    let lines: Vec<&str> = content.lines().take(n).collect();
    Ok(lines.join("\n"))
}

pub fn angles_tail(path: &str, n: usize) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("读取失败: {}", e))?;
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(n);
    Ok(lines[start..].join("\n"))
}

pub fn angles_ls(dir: &str) -> Result<String, String> {
    let dir = if dir.is_empty() { "." } else { dir };
    let entries = fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))?;
    let mut items: Vec<String> = Vec::new();
    for entry in entries {
        if let Ok(e) = entry {
            let name = e.file_name().to_string_lossy().to_string();
            let ft = e.file_type();
            let prefix = if ft.map(|t| t.is_dir()).unwrap_or(false) { "📁 " } else { "📄 " };
            items.push(format!("{}{}", prefix, name));
        }
    }
    items.sort();
    Ok(items.join("\n"))
}

pub fn angles_tree(dir: &str, depth: usize) -> Result<String, String> {
    let dir = if dir.is_empty() { "." } else { dir };
    fn walk(dir: &Path, prefix: String, depth: usize, max_depth: usize, out: &mut Vec<String>) {
        if depth > max_depth { return; }
        if let Ok(entries) = fs::read_dir(dir) {
            let mut items: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            items.sort_by_key(|e| e.file_name());
            for (i, entry) in items.iter().enumerate() {
                let last = i == items.len() - 1;
                let branch = if last { "└── " } else { "├── " };
                let name = entry.file_name().to_string_lossy().to_string();
                out.push(format!("{}{}{}", prefix, branch, name));
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let next_prefix = if last { format!("{}    ", prefix) } else { format!("{}│   ", prefix) };
                    walk(&entry.path(), next_prefix, depth + 1, max_depth, out);
                }
            }
        }
    }
    let mut out = vec![dir.to_string()];
    walk(Path::new(dir), String::new(), 0, depth, &mut out);
    Ok(out.join("\n"))
}

pub fn angles_pwd() -> Result<String, String> {
    Ok(std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| ".".to_string()))
}

pub fn angles_cd(dir: &str) -> Result<String, String> {
    std::env::set_current_dir(dir).map_err(|e| format!("切换目录失败: {}", e))?;
    Ok(format!("已切换到: {}", dir))
}

pub fn angles_fileinfo(path: &str) -> Result<String, String> {
    let meta = fs::metadata(path).map_err(|e| format!("获取信息失败: {}", e))?;
    let size = meta.len();
    let perms = if meta.permissions().readonly() { "r--r--r--" } else { "rw-r--r--" };
    let modified = meta.modified()
        .map(|t| {
            let dt: chrono::DateTime<chrono::Local> = t.into();
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_else(|_| "unknown".to_string());
    let ft = if meta.is_dir() { "directory" } else if meta.is_file() { "file" } else { "symlink" };
    Ok(format!("  路径:   {}\n  类型:   {}\n  大小:   {} bytes\n  权限:   {}\n  修改:   {}", path, ft, size, perms, modified))
}

pub fn angles_runbg(cmd: &str) -> Result<String, String> {
    let child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .spawn()
        .map_err(|e| format!("启动失败: {}", e))?;
    let pid = child.id();
    Ok(format!("后台启动 (PID={}): {}", pid, cmd))
}

pub fn angles_kill(pid: u32) -> Result<String, String> {
    let pid_str = pid.to_string();
    Command::new("kill")
        .arg(&pid_str)
        .output()
        .map_err(|e| format!("终止失败: {}", e))?;
    Ok(format!("已发送终止信号: PID={}", pid))
}

pub fn angles_fetch(url: &str, output_path: &str) -> Result<String, String> {
    let result = Command::new("curl")
        .args(["-fsSL", "-o", output_path, url])
        .output()
        .map_err(|e| format!("下载失败: {}", e))?;
    if result.status.success() {
        Ok(format!("已下载: {} → {}", url, output_path))
    } else {
        Err(format!("下载失败: {}", String::from_utf8_lossy(&result.stderr)))
    }
}

pub fn angles_gitinit(dir: &str) -> Result<String, String> {
    let dir = if dir.is_empty() { "." } else { dir };
    let output = Command::new("git").args(["init", dir]).output()
        .map_err(|e| format!("git init 失败: {}", e))?;
    if output.status.success() {
        Ok(format!("Git 仓库已初始化: {}", dir))
    } else {
        Err(format!("git init 失败: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

pub fn angles_gitcommit(msg: &str) -> Result<String, String> {
    Command::new("git").args(["add", "-A"]).output()
        .map_err(|e| format!("git add 失败: {}", e))?;
    let output = Command::new("git").args(["commit", "-m", msg]).output()
        .map_err(|e| format!("git commit 失败: {}", e))?;
    if output.status.success() {
        Ok(format!("已提交: {}", msg))
    } else {
        Err(format!("git commit 失败: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

pub fn angles_gitlog(n: usize) -> Result<String, String> {
    let n_str = format!("-{}", n);
    let output = Command::new("git").args(["log", "--oneline", &n_str]).output()
        .map_err(|e| format!("git log 失败: {}", e))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn angles_gitdiff(path: &str) -> Result<String, String> {
    let mut args = vec!["diff"];
    if !path.is_empty() { args.push(path); }
    let output = Command::new("git").args(&args).output()
        .map_err(|e| format!("git diff 失败: {}", e))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn angles_gitbranch(name: &str) -> Result<String, String> {
    Command::new("git").args(["checkout", "-b", name]).output()
        .map_err(|e| format!("git branch 失败: {}", e))?;
    Ok(format!("已创建并切换到分支: {}", name))
}

// ─── v0.6.0: 真正的网页搜索与可读文本拉取 ───

/// 抓取一个 URL 的原始 HTML（同步，走系统 curl）。失败返回 Err。
fn curl_html(url: &str) -> Result<String, String> {
    let out = Command::new("curl")
        .args([
            "-fsSL", "--max-time", "20",
            "-A", "Mozilla/5.0 (compatible; angles-cli/0.6)",
            "-L", url,
        ])
        .output()
        .map_err(|e| format!("网络请求失败: {}", e))?;
    if !out.status.success() {
        return Err(format!("无法获取 {}: HTTP {}", url, out.status));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// 把 HTML 转成"浏览器看到的可读文本"：
/// 去掉脚本/样式/注释/标签，只留正文文本，折叠多余空白。
/// 不做 CSS 渲染，因此不会出现 px、font-size 等样式噪音。
/// 把一段 HTML 剥成"读者在浏览器里看到的正文文本"：
/// 去掉 <script>/<style>/注释/所有标签，解码常用实体，折叠空白。
/// 因为直接把样式块和标签文本丢弃，返回里不会出现 px、font-size 这类 CSS 噪音。
///
/// 说明：<script> 内可能有字面 '<'，出现时保留的很少且不影响正文理解。
pub fn html_to_readable(html: &str) -> String {
    let mut s = html.to_string();

    // 1) 去注释 <-- ... -->
    strip_comment(&mut s);

    // 2) 反复剥离噪声整块：这些块内文字不是正文（script=JS，style=CSS，其余辅助），
    //    且 CSS/JS 文本里很少含 '<'，不干净时残留量小。循环直到不再变化。
    for tag in ["script", "style", "noscript", "template", "head", "iframe"] {
        strip_noise_block(&mut s, tag);
    }

    // 3) 逐字符删掉剩余全部标签，同时为块级结束标签补个换行，为单元格分隔加 ' | '。
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    let mut in_tag = false;
    let mut tag_lower = String::new();
    while let Some(c) = chars.next() {
        if in_tag {
            if c == '>' {
                in_tag = false;
                // 根据闭合标签名决定补什么节奏
                let t = tag_lower.trim();
                if t.starts_with('/') {
                    let name = t[1..].split(|ch: char| ch == ' ' || ch == '\t').next().unwrap_or("").to_lowercase();
                    match name.as_str() {
                        "p"|"div"|"li"|"tr"|"section"|"article"|"header"|"footer"|"main"
                        |"ul"|"ol"|"blockquote"|"pre"|"table"|"h1"|"h2"|"h3"|"h4"|"h5"|"h6"
                            => out.push('\n'),
                        "td"|"th" => out.push_str(" | "),
                        _ => out.push(' '),
                    }
                } else if t.starts_with("br") {
                    out.push('\n');
                } else {
                    out.push(' ');
                }
                tag_lower.clear();
            } else {
                tag_lower.push(c.to_ascii_lowercase());
            }
        } else if c == '<' {
            in_tag = true;
            tag_lower.clear();
        } else {
            out.push(c);
        }
    }
    s = out;

    // 4) 解码常用 HTML 实体
    decode_entities(&mut s);

    // 5) 折叠空白，逐行清洗
    let mut lines: Vec<String> = Vec::new();
    for raw in s.lines() {
        let coll: String = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        let t = coll.trim();
        if !t.is_empty() { lines.push(t.to_string()); }
    }
    lines.join("\n")
}

/// 去掉 HTML 注释（幂等）
fn strip_comment(s: &mut String) {
    loop {
        match (s.find("<!--"), s.find("-->")) {
            (Some(a), Some(b)) if b > a => {
                let mut n = String::new();
                n.push_str(&s[..a]);
                n.push_str(&s[b + 3..]);
                *s = n;
            }
            _ => break,
        }
    }
}

/// 去掉单个噪声标签成对的整块内容（保留闭合标签无关紧要，直接全删）。
/// 用 lower-case 比较，一次删一对。
fn strip_noise_block(s: &mut String, tag: &str) {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let mut guard = 0usize;
    loop {
        guard += 1;
        if guard > 10_000 { break; }
        let lower = s.to_lowercase();
        let Some(start) = lower.find(&open) else { break; };
        // 跳过闭合标签形态 —— 但不是开标签(前字符 '?' 或 '/')则不是我们要的块起点
        let is_open_start = {
            let before = start.checked_sub(1).map(|i| s.as_bytes()[i]).unwrap_or(0);
            before != b'/' && lower[start..].get(open.len()..).map_or(false, |rest| {
                // <tag 后应紧跟空白 / 斜杠 / >（即属性或结束）
                rest.chars().next().map_or(false, |ch| ch == ' ' || ch == '\t' || ch == '\n' || ch == '/' || ch == '>')
            })
        };
        if !is_open_start {
            // 不是可匹配的块起点（例如闭合形态或其它 tag 文字匹配）——删掉 `<tag` 前缀继续搜，避免死循环
            let mut n = String::with_capacity(s.len());
            n.push_str(&s[..start]);
            n.push_str(&s[start + open.len()..]);
            *s = n;
            continue;
        }
        // 找匹配的 close
        let Some(rel) = lower[start..].find(&close) else {
            // 找不到闭合：删除开标签本身
            let Some(after_gt) = s[start..].find('>') else { break; };
            let end = start + after_gt + 1;
            let mut n = String::new();
            n.push_str(&s[..start]);
            n.push_str(&s[end..]);
            *s = n;
            continue;
        };
        let close_mark = start + rel;
        // close_mark 指向 </tag，找其 '>'
        let after_close = s[close_mark..].find('>').map(|i| close_mark + i + 1).unwrap_or(s.len());
        let mut n = String::new();
        n.push_str(&s[..start]);
        n.push_str(&s[after_close..]);
        *s = n;
    }
}

/// 解码文本实体
fn decode_entities(s: &mut String) {
    let simple: [(&str, &str); 14] = [
        ("&nbsp;", " "), ("&amp;", "&"), ("&lt;", "<"), ("&gt;", ">"),
        ("&quot;", "\""), ("&apos;", "'"), ("&times;", "×"), ("&middot;", "·"),
        ("&mdash;", "—"), ("&ndash;", "–"), ("&copy;", "©"), ("&hellip;", "…"),
        ("&#39;", "'"), ("&#x27;", "'"),
    ];
    for &(k, v) in simple.iter() { *s = s.replace(k, v); }
}


pub fn angles_websearch_fetch(engine_url: &str) -> Result<String, String> {
    let html = curl_html(engine_url)?;
    let text = html_to_readable(&html);
    if text.trim().is_empty() {
        return Err(format!("搜索引擎未返回可解析结果（可能被反爬），链接: {}", engine_url));
    }
    // 截断，防止灌爆上下文
    let truncated: String = text.chars().take(6000).collect();
    let total = text.chars().count();
    let suffix = if total > 6000 { format!("\n…(结果过长已截断，剩余约 {} 字符。若需详情，用 angles-fetchpage 打开具体链接阅读全文)", total - 6000) } else { String::new() };
    Ok(format!("[搜索链接]\n{}\n\n[网页可读结果]\n{}{}", engine_url, truncated, suffix))
}

/// v0.6 新增工具：网页可读文本拉取器。
/// 传入 URL → 返回该页面"浏览器里看到的正文文本"，不含 HTML 标签/CSS 像素等噪音。
pub fn angles_fetchpage(url: &str, max_chars: usize) -> Result<String, String> {
    let html = curl_html(url)?;
    let mut text = html_to_readable(&html);
    if text.trim().is_empty() {
        return Err(format!("页面无可读文本（可能是 JS 动态渲染，需浏览器）。链接: {}", url));
    }
    let limit = if max_chars == 0 { 8000 } else { max_chars };
    let total = text.chars().count();
    if total > limit {
        let cut: String = text.chars().take(limit).collect();
        text = cut;
        text.push_str(&format!("\n…(正文过长已按 {} 字符截断，原文共约 {} 字符。若需更完整可用更大 max_chars)", limit, total));
    }
    Ok(text)
}
