const fs = require("node:fs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");

const SEMVER_RE = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;
const rawNextVersion = process.argv[2];
const nextVersion = rawNextVersion?.trim();

if (!nextVersion || !SEMVER_RE.test(nextVersion)) {
	console.error(
		"Error: Invalid or missing version (expected SemVer, e.g., 1.2.3)",
	);
	process.exit(1);
}

console.log(`🚀 Updating project versions to ${nextVersion}...`);

// Root of the repository. This script lives at npm/organiza/scripts/update-versions.js,
// so the repo root is three levels up.
const rootDir = path.resolve(__dirname, "..", "..", "..");
let hadErrors = false;

// 1. Update Cargo.toml (use file descriptor to avoid TOCTOU)
const cargoPath = path.join(rootDir, "Cargo.toml");
try {
	const fd = fs.openSync(cargoPath, "r+");
	try {
		const cargoContent = fs.readFileSync(fd, "utf8");
		let versionUpdated = false;

		// Robust regex to find version inside [package] or [workspace.package]
		const updatedCargo = cargoContent.replace(
			/(\[(?:workspace\.)?package\][\s\S]*?^\s*version\s*=\s*")([^"]*)(")/m,
			(_match, prefix, oldVersion, suffix) => {
				versionUpdated = true;
				console.log(
					`  Found Cargo.toml version: ${oldVersion} inside package section`,
				);
				return `${prefix}${nextVersion}${suffix}`;
			},
		);

		if (!versionUpdated) {
			console.error(
				"❌ Could not find version line inside [package] or [workspace.package] section of Cargo.toml",
			);
			// Log the first 200 chars to debug
			console.log("--- Cargo.toml content start ---");
			console.log(cargoContent.substring(0, 200));
			console.log("--- End ---");
			hadErrors = true;
		} else {
			// Truncate and write using the same file descriptor to avoid races
			fs.ftruncateSync(fd, 0);
			fs.writeSync(fd, updatedCargo, 0, "utf8");
			fs.fsyncSync(fd);
			console.log("✅ Updated Cargo.toml");
		}
	} finally {
		try {
			fs.closeSync(fd);
		} catch (_e) {
			/* best-effort close */
		}
	}
} catch (err) {
	if (err && err.code === "ENOENT") {
		// File doesn't exist; nothing to do
	} else if (err) {
		console.error("❌ Failed updating Cargo.toml:", err.message || err);
		hadErrors = true;
	}
}

// 2. Root package.json is intentionally absent (ADR-2: plain npm, no root
// workspace package). Nothing to update here — release-please keeps every
// artifact on one version via extra-files jsonpaths.

// 3. Update npm/organiza/package.json (use file descriptor to avoid TOCTOU)
const npmPkgPath = path.join(rootDir, "npm/organiza/package.json");
const npmPkgDir = path.join(rootDir, "npm/organiza");
try {
	const fd = fs.openSync(npmPkgPath, "r+");
	try {
		const current = fs.readFileSync(fd, "utf8");
		const npmPkg = JSON.parse(current);
		npmPkg.version = nextVersion;
		const newContents = `${JSON.stringify(npmPkg, null, 2)}\n`;

		fs.ftruncateSync(fd, 0);
		fs.writeSync(fd, newContents, 0, "utf8");
		fs.fsyncSync(fd);
		console.log("✅ Updated npm/organiza/package.json");
	} finally {
		try {
			fs.closeSync(fd);
		} catch (_e) {
			/* best-effort close */
		}
	}
} catch (err) {
	if (err && err.code === "ENOENT") {
		// File doesn't exist; skip
	} else if (err) {
		console.error(
			"❌ Failed updating npm/organiza/package.json:",
			err.message || err,
		);
		hadErrors = true;
	}
}

// 4. Run sync-optional-deps.js if it exists
const syncOptionalDepsPath = path.join(
	npmPkgDir,
	"scripts/sync-optional-deps.js",
);
if (fs.existsSync(syncOptionalDepsPath)) {
	console.log("🔄 Running sync-optional-deps.js...");
	try {
		// We run it from the npm/organiza directory using execFileSync for safety
		execFileSync(
			process.execPath,
			["scripts/sync-optional-deps.js", nextVersion],
			{
				cwd: npmPkgDir,
				stdio: "inherit",
			},
		);
		console.log("✅ Updated optional dependencies");
	} catch (error) {
		console.error("❌ Error running sync-optional-deps.js:", error.message);
		hadErrors = true;
	}
}

if (hadErrors) {
	console.error("❌ Version update completed with errors.");
	process.exit(1);
}

console.log("🎉 All versions updated successfully!");
