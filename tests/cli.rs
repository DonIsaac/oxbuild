use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

struct TestProject {
    root: PathBuf,
}

impl TestProject {
    fn new(name: &str) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = env::temp_dir().join(format!("oxbuild-{name}-{}-{id}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn file(self, path: impl AsRef<Path>, contents: &str) -> Self {
        let path = self.root.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
        self
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for TestProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run_oxbuild(project: &TestProject, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_oxbuild"))
        .current_dir(project.path())
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn compiles_default_src_to_dist() {
    let project = TestProject::new("default-src")
        .file("package.json", r#"{"name":"fixture"}"#)
        .file("src/index.ts", "export const answer: number = 42;\n");

    let output = run_oxbuild(&project, &[]);

    assert!(
        output.status.success(),
        "oxbuild failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let js = fs::read_to_string(project.path().join("dist/index.js")).unwrap();
    assert!(js.contains("const answer = 42"));
    assert!(project.path().join("dist/index.js.map").exists());
}

#[test]
fn respects_tsconfig_root_dir_out_dir_and_isolated_declarations() {
    let project = TestProject::new("tsconfig")
        .file("package.json", r#"{"name":"fixture"}"#)
        .file(
            "tsconfig.json",
            r#"{
  "compilerOptions": {
    "rootDir": "source",
    "outDir": "build",
    "isolatedDeclarations": true
  }
}"#,
        )
        .file(
            "source/index.ts",
            "export interface User { name: string }\nexport const user: User = { name: 'Ada' };\n",
        );

    let output = run_oxbuild(&project, &[]);

    assert!(
        output.status.success(),
        "oxbuild failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(project.path().join("build/index.js").exists());
    let declarations = fs::read_to_string(project.path().join("build/index.d.ts")).unwrap();
    assert!(declarations.contains("interface User"));
}

#[test]
fn reports_parse_errors_with_nonzero_exit() {
    let project = TestProject::new("parse-error")
        .file("package.json", r#"{"name":"fixture"}"#)
        .file("src/index.ts", "export const broken = ;\n");

    let output = run_oxbuild(&project, &[]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("index.ts") || stderr.contains("broken"),
        "expected diagnostic context in stderr, got:\n{stderr}"
    );
}
