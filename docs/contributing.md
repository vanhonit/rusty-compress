---
layout: page
title: Contributing
subtitle: Help improve Rusty Compress
---

## Welcome Contributors! 🎉

Thank you for considering contributing to Rusty Compress! We appreciate any help you can provide, whether it's reporting bugs, fixing issues, adding features, or improving documentation.

## Ways to Contribute

### 🐛 Report Bugs

Found a bug? Help us fix it!

1. **Search existing issues** - Check if the bug has already been reported
2. **Create a detailed report** - Include:
   - Steps to reproduce
   - Expected vs actual behavior
   - Screenshots (if applicable)
   - System information (OS, app version)
3. **Use appropriate labels** - Bug, UI, Backend, etc.

### 💡 Suggest Features

Have an idea for improvement?

1. **Check roadmap** - See if it's already planned
2. **Create a feature request** - Describe:
   - The problem you're solving
   - Why this feature is useful
   - Potential implementation approaches
3. **Discuss alternatives** - Different ways to solve the problem

### 🔒 Security Issues

Found a security vulnerability?

1. **Don't use public issues** - Email security concerns privately
2. **Provide details** - Describe the vulnerability and potential impact
3. **Allow time to fix** - Give us time to develop and test a fix

### 📖 Improve Documentation

Documentation is crucial!

- **Fix typos** - Correct spelling and grammar mistakes
- **Add examples** - Provide code examples and use cases
- **Clarify sections** - Make complex topics easier to understand
- **Translate** - Help translate documentation to other languages

### 💻 Code Contributions

Ready to write code? Great!

## Getting Started

### 1. Set Up Development Environment

```bash
# Fork the repository
# Clone your fork
git clone https://github.com/YOUR_USERNAME/rusty-compress.git
cd rusty-compress

# Install dependencies
npm install

# Set up development environment
npm run tauri dev
```

### 2. Choose What to Work On

- **Good First Issues** - Start with issues marked as "good first issue"
- **Help Wanted** - Issues needing community help
- **Feature Requests** - Implement requested features

### 3. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 4. Make Your Changes

Follow our code style and patterns:
- Use existing component patterns
- Write clear, descriptive comments
- Add tests for new functionality
- Update documentation

### 5. Test Your Changes

```bash
# Run tests (Rust unit tests; there are no frontend tests)
cd src-tauri && cargo test

# Run linter
cd src-tauri && cargo clippy --all-targets

# Build project
npm run build
npm run tauri build
```

### 6. Commit Your Changes

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
git add .
git commit -m "feat: add support for 7z archives"
# or
git commit -m "fix: resolve extraction progress issue"
```

### 7. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a PR on GitHub with:
- Clear title and description
- Reference related issues
- Screenshots for UI changes
- Testing instructions

## Coding Standards

### General Guidelines

- **Be consistent** - Follow existing code patterns
- **Keep it simple** - Simple code is easier to maintain
- **Test thoroughly** - Write tests for new functionality
- **Document clearly** - Add helpful comments and documentation

### Frontend (React)

```jsx
// ✅ Good
const FileManager = ({ currentPath, onNavigate }) => {
  const [files, setFiles] = useState([]);
  
  useEffect(() => {
    loadFiles(currentPath);
  }, [currentPath]);

  return <FileList files={files} onSelect={handleFileSelect} />;
};

// ❌ Bad
const FileManager = (props) => {
  let [files, setFiles] = useState([]);
  
  // Missing dependencies
  useEffect(() => {
    loadFiles(props.currentPath);
  }, []);
  
  return <div>{files.map(f => <div>{f.name}</div>)}</div>;
};
```

### Backend (Rust)

```rust
// ✅ Good
#[tauri::command]
async fn read_directory(path: String) -> Result<Vec<FileInfo>, String> {
    tokio::spawn_blocking(move || {
        fs::read_dir(&path)
            .map_err(|e| format!("Failed to read directory: {}", e))?
            .filter_map(|entry| entry.ok())
            .map(|entry| FileInfo::from(entry))
            .collect()
    }).await.map_err(|e| e.to_string())?
}

// ❌ Bad
#[tauri::command]
async fn read_directory(p: String) -> Result<Vec<FileInfo>> {
    let entries = fs::read_dir(p).unwrap();
    Ok(entries.collect())
}
```

### Error Handling

```jsx
// Frontend error handling
const handleSubmit = async () => {
  try {
    const result = await invoke('extract_files', params);
    showSuccessNotification('Extraction complete!');
  } catch (error) {
    console.error('Extraction failed:', error);
    showErrorNotification(error.message || 'Unknown error');
  }
};
```

```rust
// Backend error handling
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

## Testing

### Frontend Tests

```jsx
// Component test
describe('FileManager', () => {
  test('displays file list', () => {
    const files = [
      { name: 'file1.txt', size: 1024 },
      { name: 'file2.txt', size: 2048 }
    ];
    
    render(<FileManager files={files} />);
    
    expect(screen.getByText('file1.txt')).toBeInTheDocument();
    expect(screen.getByText('file2.txt')).toBeInTheDocument();
  });
});
```

### Backend Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_extract_files() {
        let result = extract_files(
            &app_handle(),
            "test.zip",
            "/tmp/output",
            vec!["file.txt"]
        ).await;
        
        assert!(result.is_ok());
    }
}
```

## Documentation

### Code Documentation

```rust
/// Extracts files from an archive
///
/// # Arguments
/// * `archive_path` - Path to the archive file
/// * `output_path` - Destination directory for extracted files
/// * `selected_files` - List of files to extract (empty = all)
///
/// # Returns
/// Result indicating success or error
///
/// # Examples
/// ```rust
/// extract_files(&app, "archive.zip", "/tmp", vec!["file.txt"]).await
/// ```
#[tauri::command]
async fn extract_files(
    app_handle: AppHandle,
    archive_path: String,
    output_path: String,
    selected_files: Vec<String>
) -> Result<(), String> {
    // Implementation
}
```

## Pull Request Guidelines

### Title Format

```
feat: add support for 7z archives
fix: resolve extraction progress issue
docs: update installation guide
refactor: simplify file manager component
test: add integration tests for checksum computation
```

### Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Related Issues
Fixes #123
Related to #456

## Testing
- [ ] Tests pass locally
- [ ] Added new tests
- [ ] Tested manually

## Screenshots (if applicable)
[Attach screenshots]

## Checklist
- [ ] Code follows style guidelines
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Commit messages follow guidelines
```

## Code Review Process

### Before Submitting

1. **Self-review** - Review your own changes
2. **Run tests** - Ensure all tests pass
3. **Update docs** - Keep documentation in sync
4. **Check formatting** - Run linter and formatter

### During Review

1. **Be responsive** - Address feedback promptly
2. **Ask questions** - Clarify review comments
3. **Iterate** - Make requested changes
4. **Stay patient** - Reviews take time

### After Merging

1. **Update branches** - Sync with upstream
2. **Celebrate** - Your contribution is live!
3. **Keep contributing** - There's always more to do

## Recognition

Contributors will be recognized in:
- **CONTRIBUTORS.md** - List of all contributors
- **Release Notes** - Credits for each release
- **GitHub Stats** - Contribution statistics

## Community Guidelines

### Be Respectful

- Treat others with respect and kindness
- Welcome newcomers and help them learn
- Assume good intentions
- Focus on what's best for the community

### Be Constructive

- Provide helpful, actionable feedback
- Explain reasoning clearly
- Suggest improvements
- Acknowledge good work

### Be Inclusive

- Use inclusive language
- Consider diverse perspectives
- Make documentation accessible
- Support users of all skill levels

## Communication Channels

- **GitHub Issues** - Bug reports and feature requests
- **Pull Requests** - Code contributions and reviews
- **Discussions** - Questions and ideas
- **Email** - For security concerns and private matters

## Resources for Contributors

### Learning Resources

- [React Documentation](https://react.dev/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Tauri Guide](https://tauri.app/v1/guides/)
- [Contributing to Open Source](https://opensource.guide/how-to-contribute/)

### Project Resources

- [Project README](../README.md)
- [Architecture Documentation](../CLAUDE.md)
- [API Reference](api.html)
- [Development Guide](development.html)

## Getting Help

### Stuck? Ask for Help!

- **Comment on issues** - Ask for clarification
- **Join discussions** - Get community input
- **Mention maintainers** - Get expert help
- **Search documentation** - Find existing answers

### No Question is Too Basic

We were all beginners once! Don't hesitate to ask questions about:
- Setup and installation
- Code structure and patterns
- Testing and debugging
- Contribution process

## Impact of Contributions

Your contributions help make Rusty Compress:

- **More reliable** - Bug fixes improve stability
- **More capable** - New features add functionality
- **More accessible** - Better docs help more users
- **More popular** - Quality improvements attract users

## Thank You!

Every contribution, no matter how small, makes a difference. Thank you for helping make Rusty Compress better for everyone!

---

<div class="card" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; border: none; margin-top: 2rem;">
    <div class="card-header">
        <h3 class="card-title" style="color: white;">🎯 Ready to Make Your First Contribution?</h3>
    </div>
    <p style="color: rgba(255,255,255,0.9);">Start by finding a "good first issue" or helping improve documentation. Every contribution counts!</p>
    <a href="https://github.com/vanhonit/rusty-compress/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22" class="btn" style="background: white; color: #667eea;">Find Good First Issues</a>
    <a href="https://github.com/vanhonit/rusty-compress/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22" class="btn" style="background: white; color: #667eea; margin-left: 0.5rem;">Help Wanted</a>
</div>