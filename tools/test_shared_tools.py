# =============================================================================
# EH3D-MANUFACTURING-PROFILE - tools/test_shared_tools.py
# Copyright (C) 2026 JuanenRac (Electro Hobby 3D) <electrohobby3d@gmail.com>
# GPL-3.0-or-later - see LICENSE
# =============================================================================
"""The shared tools of the repository (the lessons linter, the workflow check, the CHANGELOG check of a pull request and the banner check) do what their
documentation says on small inputs made by the test itself: the findings, the baselines, the exit codes and the shapes they print stay as they are."""
from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parent


def load(name: str):
    """One tool of this folder as a module, loaded from its path so that the test does not depend on how it was started."""
    spec = importlib.util.spec_from_file_location(f"shared_{name}", TOOLS / f"{name}.py")
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


LL = load("lessons_lint")
WH = load("workflow_hygiene")
CP = load("changelog_pr_check")
BC = load("banner_check")


def tree_of(source: str):
    import ast

    return ast.parse(source), source.split("\n")


class Folder(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.root = Path(self._tmp.name)

    def put(self, name: str, text: str) -> Path:
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        return path


class LessonsLintTests(Folder):
    def test_a_call_to_subprocess_without_a_timeout_is_found_on_its_line(self) -> None:
        tree, lines = tree_of("import subprocess\nsubprocess.run(['x'])\n")
        self.assertEqual([n for n, _ in LL.lint_subprocess(tree, lines)], [2])

    def test_a_call_with_a_timeout_or_with_unpacked_keywords_is_not_found(self) -> None:
        tree, lines = tree_of("import subprocess\nsubprocess.run(['x'], timeout=5)\nsubprocess.call(['y'], **kw)\n")
        self.assertEqual(LL.lint_subprocess(tree, lines), [])

    def test_the_pragma_of_the_line_lets_a_call_without_a_timeout_pass(self) -> None:
        tree, lines = tree_of("import subprocess\nsubprocess.check_output(['x'])  # lessons: ok\n")
        self.assertEqual(LL.lint_subprocess(tree, lines), [])

    def test_a_function_imported_by_name_and_renamed_is_still_found(self) -> None:
        tree, lines = tree_of("from subprocess import check_output as co\nco(['x'])\n")
        found = LL.lint_subprocess(tree, lines)
        self.assertEqual(len(found), 1)
        self.assertIn("co(...)", found[0][1])

    def test_a_call_of_another_module_with_the_same_name_is_not_found(self) -> None:
        tree, lines = tree_of("import os\nos.run(['x'])\nrun(['y'])\n")
        self.assertEqual(LL.lint_subprocess(tree, lines), [])

    def test_a_sentence_printed_outside_a_catalogue_is_found_and_a_key_is_not(self) -> None:
        tree, lines = tree_of("print('the file was written here')\nprint(t('done'))\nprint('ok')\n")
        self.assertEqual([n for n, _ in LL.lint_visible(tree, lines)], [1])

    def test_the_help_text_of_an_argument_counts_as_visible_text(self) -> None:
        tree, lines = tree_of("p.add_argument('--x', help='what the option does here')\n")
        self.assertEqual(len(LL.lint_visible(tree, lines)), 1)

    def test_imports_i18n_sees_the_plain_and_the_from_import_and_nothing_else(self) -> None:
        self.assertTrue(LL.imports_i18n(tree_of("import i18n\n")[0]))
        self.assertTrue(LL.imports_i18n(tree_of("from pkg import i18n_tools\n")[0]))
        self.assertTrue(LL.imports_i18n(tree_of("from pkg.i18n import t\n")[0]))
        self.assertFalse(LL.imports_i18n(tree_of("import json\n")[0]))

    def test_flatten_gives_the_dotted_keys_and_skips_the_top_level_private_ones(self) -> None:
        keys = list(LL.flatten({"a": {"b": 1, "c": {"d": 2}}, "_note": "x", "e": 3}))
        self.assertEqual(sorted(keys), ["a.b", "a.c.d", "e"])

    def test_the_catalogue_of_a_missing_language_and_of_a_missing_key_are_reported(self) -> None:
        self.put("lang/en.json", json.dumps({"a": "x", "b": "y", "_optional": ["b"]}))
        self.put("lang/es.json", json.dumps({"a": "x"}))
        self.put("lang/de.json", json.dumps({"b": "y"}))
        found = LL.lint_lang_keys(self.root, [self.root / "lang"])
        by_file = {f["file"]: f["message"] for f in found}
        self.assertIn("lang/de.json", by_file)
        self.assertIn("1 keys of en.json missing: a", by_file["lang/de.json"])
        self.assertNotIn("lang/es.json", by_file)
        self.assertIn("missing file for the language 'fr'", by_file["lang/fr.json"])
        self.assertEqual(sum(1 for f in found if "missing file" in f["message"]), 5)

    def test_an_english_catalogue_that_is_not_json_is_one_finding_and_stops_there(self) -> None:
        self.put("lang/en.json", "{not json")
        found = LL.lint_lang_keys(self.root, [self.root / "lang"])
        self.assertEqual([f["message"] for f in found], ["en.json is not valid JSON"])

    def test_the_package_with_the_name_of_the_repository_is_its_own_and_a_library_is_not(self) -> None:
        self.assertTrue(LL.is_own_package("URTC-GATEWAY", "urtc_gateway"))
        self.assertTrue(LL.is_own_package("EH3D-DESIGN-VERIFIER", "design_verifier"))
        self.assertFalse(LL.is_own_package("URTC-FLASHER", "urtc_can"))

    def test_the_key_of_a_counted_rule_ignores_the_message_and_the_others_keep_it(self) -> None:
        counted = {"rule": "subprocess-timeout", "file": "a.py", "line": 1, "message": "one"}
        other = {"rule": "visible-text", "file": "a.py", "line": 1, "message": "two"}
        self.assertEqual(LL.finding_key(counted), "subprocess-timeout|a.py|")
        self.assertEqual(LL.finding_key(other), "visible-text|a.py|two")

    def test_a_baseline_that_is_missing_or_damaged_is_empty(self) -> None:
        self.assertEqual(sum(LL.load_baseline(self.root / "none.json").values()), 0)
        bad = self.put("bad.json", "[1, 2")
        self.assertEqual(sum(LL.load_baseline(bad).values()), 0)
        wrong = self.put("wrong.json", json.dumps({"findings": [1, 2]}))
        self.assertEqual(sum(LL.load_baseline(wrong).values()), 0)

    def test_a_baseline_written_and_read_again_gives_the_same_counts_in_the_same_order(self) -> None:
        result = {"findings": [{"rule": "visible-text", "file": "b.py", "line": 3, "message": "m"}, {"rule": "subprocess-timeout", "file": "a.py", "line": 1, "message": "x"},
                               {"rule": "subprocess-timeout", "file": "a.py", "line": 9, "message": "y"}]}
        path = self.root / "tools" / "base.json"
        LL.write_baseline(result, path)
        first = path.read_text(encoding="utf-8")
        loaded = LL.load_baseline(path)
        self.assertEqual(loaded["subprocess-timeout|a.py|"], 2)
        self.assertEqual(loaded["visible-text|b.py|m"], 1)
        LL.write_baseline(result, path)
        self.assertEqual(path.read_text(encoding="utf-8"), first)
        self.assertTrue(first.endswith("\n"))
        keys = list(json.loads(first)["findings"])
        self.assertEqual(keys, sorted(keys))

    def test_a_finding_is_ignored_by_the_glob_of_its_rule_or_of_every_rule(self) -> None:
        item = {"rule": "visible-text", "file": "tools/x.py", "line": 1, "message": "m"}
        self.assertTrue(LL.is_ignored(item, {"visible-text": ["tools/*.py"]}))
        self.assertTrue(LL.is_ignored(item, {"*": ["tools/*"]}))
        self.assertFalse(LL.is_ignored(item, {"visible-text": ["src/*"], "lang-keys": ["tools/*"]}))
        self.assertFalse(LL.is_ignored(item, {}))

    def test_the_added_lines_of_a_diff_come_with_their_numbers_and_a_deleted_file_has_none(self) -> None:
        diff = "\n".join(["--- a/x.py", "+++ b/x.py", LL.HUNK + " -1,0 +5,2 " + LL.HUNK, "+first", "+second", "--- a/gone.py", "+++ /dev/null", LL.HUNK + " -1 +0,0 " + LL.HUNK, "-old"])
        self.assertEqual(LL.parse_diff(diff), {"x.py": {5: "first", 6: "second"}})

    def test_the_walk_skips_the_private_folders_and_gives_only_the_wanted_suffixes(self) -> None:
        self.put("a.py", "")
        self.put("sub/b.py", "")
        self.put("sub/c.txt", "")
        self.put("node_modules/d.py", "")
        self.put("target/e.py", "")
        names = sorted(p.relative_to(self.root).as_posix() for p in LL.walk(self.root, (".py",)))
        self.assertEqual(names, ["a.py", "sub/b.py"])

    def test_a_finding_outside_the_repository_keeps_its_whole_path(self) -> None:
        inside = LL.finding("r", self.root / "a" / "b.py", 3, "m", self.root)
        self.assertEqual(inside["file"], "a/b.py")
        with tempfile.TemporaryDirectory() as other:
            outside = LL.finding("r", Path(other) / "c.py", 1, "m", self.root)
        self.assertTrue(outside["file"].endswith("c.py"))

    def test_a_file_with_a_byte_order_mark_is_read_without_it_and_a_bad_byte_does_not_stop_the_read(self) -> None:
        path = self.root / "bom.txt"
        path.write_bytes(b"\xef\xbb\xbfhello \xff")
        text = LL.read(path)
        self.assertTrue(text.startswith("hello"))

    def test_the_section_of_a_configuration_that_is_missing_or_not_an_object_is_empty(self) -> None:
        self.assertEqual(LL.load_section(self.root, "tools/none.json"), {})
        self.put("c1.json", "[]")
        self.assertEqual(LL.load_section(self.root, "c1.json"), {})
        self.put("c2.json", json.dumps({"lessons_lint": {"enabled": False}}))
        self.assertEqual(LL.load_section(self.root, "c2.json"), {"enabled": False})
        self.assertEqual(LL.load_section(self.root, None), {})

    def test_the_pattern_of_the_private_references_finds_a_code_of_a_round_and_not_a_short_number(self) -> None:
        self.assertIsNotNone(LL.PRIVATE_RE.search("see AR1234 for it"))
        self.assertIsNone(LL.PRIVATE_RE.search("see AR12 for it"))
        self.assertIsNone(LL.PRIVATE_RE.search("a plain public sentence"))

    def test_every_rule_is_in_the_list_of_all_the_rules_and_the_names_do_not_repeat(self) -> None:
        self.assertEqual(len(LL.ALL_RULES), len(set(LL.ALL_RULES)))
        for rule in LL.RULES + LL.FILE_RULES + LL.PRECHECK_RULES:
            self.assertIn(rule, LL.ALL_RULES)
        self.assertEqual(LL.LANGS, ("en", "es", "de", "fr", "it", "ja", "zh", "pt"))


class WorkflowHygieneTests(Folder):
    PINNED = "a" * 40

    def rules(self, text: str, speed: bool = False) -> list[str]:
        return [f["rule"] for f in WH.check_workflow(Path("ci.yml"), text, speed=speed)]

    def test_the_ecosystem_comes_from_the_start_of_the_name_of_the_repository(self) -> None:
        self.assertEqual(WH.ecosystem("eh3d-thermal-lab"), "EH3D")
        self.assertEqual(WH.ecosystem("ELECTRO-HOBBY-3D-X"), "EH3D")
        self.assertEqual(WH.ecosystem("HYDRA-UMC"), "HYDRA")
        self.assertEqual(WH.ecosystem("urtc-web"), "URTC")
        self.assertEqual(WH.ecosystem("armor-server"), "ARMOR")
        self.assertEqual(WH.ecosystem("something"), "OTHER")

    def test_a_comment_is_cut_but_a_hash_inside_quotes_is_kept(self) -> None:
        self.assertEqual(WH.strip_comment("key: value  # note"), "key: value")
        self.assertEqual(WH.strip_comment('key: "a # b"  # note'), 'key: "a # b"')
        self.assertEqual(WH.strip_comment("# whole line"), "")
        self.assertEqual(WH.strip_comment("url: a#b"), "url: a#b")

    def test_an_action_by_tag_is_found_and_one_by_commit_or_of_the_owner_or_local_is_not(self) -> None:
        text = "permissions: {}\njobs:\n  a:\n    timeout-minutes: 5\n    steps:\n      - uses: actions/checkout@v4\n"
        self.assertIn("unpinned-action", self.rules(text))
        for ok in (f"actions/checkout@{self.PINNED}", "juanenrac/tool@v1", "./local/action"):
            self.assertNotIn("unpinned-action", self.rules(text.replace("actions/checkout@v4", ok)), ok)

    def test_an_action_without_a_version_and_an_image_without_a_digest_are_found(self) -> None:
        base = "permissions: {}\njobs:\n  a:\n    timeout-minutes: 5\n    steps:\n      - uses: X\n"
        self.assertIn("unpinned-action", self.rules(base.replace("X", "actions/checkout")))
        self.assertIn("unpinned-action", self.rules(base.replace("X", "docker://alpine:3")))
        self.assertNotIn("unpinned-action", self.rules(base.replace("X", "docker://alpine@sha256:" + "0" * 64)))

    def test_a_workflow_without_permissions_is_found_and_with_them_is_not(self) -> None:
        body = "jobs:\n  a:\n    timeout-minutes: 5\n    steps:\n      - run: echo ok\n"
        self.assertIn("no-permissions", self.rules(body))
        self.assertNotIn("no-permissions", self.rules("permissions:\n  contents: read\n" + body))

    def test_a_job_without_a_timeout_is_found_but_a_job_that_calls_a_reusable_workflow_is_not(self) -> None:
        plain = "permissions: {}\njobs:\n  a:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo ok\n"
        self.assertIn("no-timeout", self.rules(plain))
        reusable = "permissions: {}\njobs:\n  a:\n    uses: owner/repo/.github/workflows/x.yml@" + self.PINNED + "\n"
        self.assertNotIn("no-timeout", self.rules(reusable))

    def test_a_secret_written_in_a_run_line_or_block_is_found_and_one_in_env_is_not(self) -> None:
        head = "permissions: {}\njobs:\n  a:\n    timeout-minutes: 5\n    steps:\n"
        inline = head + "      - run: echo ${{ secrets.TOKEN_X }}\n"
        block = head + "      - run: |\n          echo start\n          echo ${{ secrets.TOKEN_X }}\n"
        safe = head + "      - run: echo \"$V\"\n        env:\n          V: ${{ secrets.TOKEN_X }}\n"
        self.assertIn("secret-in-run", self.rules(inline))
        self.assertIn("secret-in-run", self.rules(block))
        self.assertNotIn("secret-in-run", self.rules(safe))

    def test_the_line_of_a_secret_in_a_block_is_the_line_of_the_text_and_not_of_the_key(self) -> None:
        text = "permissions: {}\njobs:\n  a:\n    timeout-minutes: 5\n    steps:\n      - run: |\n          echo start\n          echo ${{ secrets.TOKEN_X }}\n"
        found = [f for f in WH.check_workflow(Path("ci.yml"), text) if f["rule"] == "secret-in-run"]
        self.assertEqual([f["line"] for f in found], [8])

    def test_a_pull_request_target_that_checks_out_the_head_of_the_request_is_found(self) -> None:
        text = "on: pull_request_target\npermissions: {}\njobs:\n  a:\n    timeout-minutes: 5\n    steps:\n      - uses: actions/checkout@" + self.PINNED + "\n        with:\n          ref: ${{ github.event.pull_request.head.sha }}\n"
        self.assertIn("prt-checkout", self.rules(text))
        self.assertNotIn("prt-checkout", self.rules(text.replace("pull_request_target", "pull_request")))

    def test_a_comment_line_never_makes_a_finding(self) -> None:
        text = "permissions: {}\njobs:\n  a:\n    timeout-minutes: 5\n    steps:\n      # - uses: actions/checkout@v4\n      - run: echo ok # ${{ secrets.X }}\n"
        self.assertEqual(self.rules(text), [])

    def test_the_events_of_a_workflow_are_read_in_the_scalar_the_list_and_the_map_forms(self) -> None:
        self.assertEqual(WH.Workflow("on: push\n").triggers(), {"push"})
        self.assertEqual(WH.Workflow("on: [push, pull_request]\n").triggers(), {"push", "pull_request"})
        self.assertEqual(WH.Workflow("on:\n  push:\n  workflow_dispatch:\n").triggers(), {"push", "workflow_dispatch"})
        self.assertEqual(WH.Workflow("name: x\n").triggers(), set())

    def test_the_jobs_are_listed_with_their_line_and_their_keys(self) -> None:
        wf = WH.Workflow("name: x\njobs:\n  build:\n    runs-on: a\n    timeout-minutes: 3\n  test:\n    runs-on: b\n")
        jobs = wf.jobs()
        self.assertEqual([j["name"] for j in jobs], ["build", "test"])
        self.assertEqual(jobs[0]["line"], 3)
        self.assertIn("timeout-minutes", jobs[0]["keys"])
        self.assertNotIn("timeout-minutes", jobs[1]["keys"])
        self.assertEqual(WH.Workflow("name: x\n").jobs(), [])

    def test_a_workflow_is_a_release_by_its_file_name_its_event_or_its_commands(self) -> None:
        self.assertTrue(WH.Workflow("name: x\n", "release.yml").is_release())
        self.assertTrue(WH.Workflow("on: release\n", "x.yml").is_release())
        self.assertTrue(WH.Workflow("jobs:\n  a:\n    steps:\n      - run: gh release create v1\n", "x.yml").is_release())
        self.assertFalse(WH.Workflow("on: push\njobs:\n  a:\n    steps:\n      - run: echo ok\n", "ci.yml").is_release())

    def test_a_release_workflow_without_concurrency_is_found_and_with_it_is_not(self) -> None:
        text = "name: Release\non: push\npermissions: {}\njobs:\n  a:\n    timeout-minutes: 5\n    steps:\n      - run: echo ok\n"
        self.assertIn("no-concurrency", self.rules(text))
        self.assertNotIn("no-concurrency", self.rules(text.replace("permissions: {}", "permissions: {}\nconcurrency: x")))

    def test_the_speed_rules_only_run_when_asked_for(self) -> None:
        text = "on: pull_request\npermissions: {}\njobs:\n  a:\n    runs-on: macos-14\n    timeout-minutes: 5\n    steps:\n      - run: echo ok\n"
        self.assertNotIn("macos-on-pr", self.rules(text))
        self.assertIn("macos-on-pr", self.rules(text, speed=True))

    def test_every_rule_has_a_severity_and_the_speed_rules_are_among_them(self) -> None:
        self.assertEqual(set(WH.RULES), set(WH.SEVERITY))
        self.assertTrue(WH.SPEED_RULES <= set(WH.RULES))
        self.assertTrue(set(WH.SEVERITY.values()) <= set(WH.WEIGHT))

    def test_the_score_adds_the_weight_of_each_severity(self) -> None:
        self.assertEqual(WH.score([]), 0)
        self.assertEqual(WH.score([{"severity": "high"}, {"severity": "medium"}, {"severity": "low"}]), 6)

    def test_the_filter_keeps_the_rules_asked_for_and_the_severity_from_the_minimum_up(self) -> None:
        items = [{"rule": "no-timeout", "severity": "low"}, {"rule": "unpinned-action", "severity": "medium"}, {"rule": "secret-in-run", "severity": "high"}]
        repos = {"r": {"findings": list(items)}}
        WH.filter_findings(repos, min_severity="medium")
        self.assertEqual([f["rule"] for f in repos["r"]["findings"]], ["unpinned-action", "secret-in-run"])
        repos = {"r": {"findings": list(items)}}
        WH.filter_findings(repos, rules={"no-timeout"})
        self.assertEqual([f["rule"] for f in repos["r"]["findings"]], ["no-timeout"])

    def test_the_summary_counts_by_ecosystem_and_by_rule_and_names_the_worst_repository_first(self) -> None:
        repos = {
            "EH3D-A": {"ecosystem": "EH3D", "workflows": 2, "findings": [{"rule": "no-timeout", "severity": "low"}]},
            "EH3D-B": {"ecosystem": "EH3D", "workflows": 1, "findings": [{"rule": "secret-in-run", "severity": "high"}, {"rule": "no-timeout", "severity": "low"}]},
            "HYDRA-C": {"ecosystem": "HYDRA", "workflows": 4, "findings": []},
        }
        summary = WH.summarize(repos)
        self.assertEqual(summary["total_repos"], 3)
        self.assertEqual(summary["total_workflows"], 7)
        self.assertEqual(summary["total_findings"], 3)
        self.assertEqual(summary["by_rule"], {"no-timeout": 2, "secret-in-run": 1})
        self.assertEqual(summary["ecosystems"]["EH3D"]["score"], 5)
        self.assertEqual([w["repo"] for w in summary["worst"]], ["EH3D-B", "EH3D-A"])

    def test_the_identity_of_a_finding_does_not_hold_its_line(self) -> None:
        a = {"file": "ci.yml", "rule": "no-timeout", "line": 3, "message": "m"}
        b = dict(a, line=30)
        self.assertEqual(WH.finding_key(a), WH.finding_key(b))

    def test_a_baseline_or_a_configuration_that_is_damaged_reads_as_empty(self) -> None:
        bad = self.put("bad.json", "{oops")
        self.assertEqual(sum(WH.load_baseline(bad).values()), 0)
        self.assertEqual(sum(WH.load_baseline(self.root / "none.json").values()), 0)
        self.assertEqual(WH.repo_section(self.root, "tools/none.json"), {})
        self.put("cfg.json", json.dumps({"workflow_hygiene": {"threshold": 3}}))
        self.assertEqual(WH.repo_section(self.root, "cfg.json"), {"threshold": 3})

    def test_the_baseline_of_a_repository_is_in_the_default_place_unless_it_is_named(self) -> None:
        self.assertEqual(WH.baseline_path(self.root, {}), self.root / WH.DEFAULT_BASELINE)
        self.assertEqual(WH.baseline_path(self.root, {"baseline": "x/b.json"}), self.root / "x" / "b.json")
        self.assertEqual(WH.baseline_path(self.root, {"baseline": "x/b.json"}, "o.json"), self.root / "o.json")


class ChangelogCheckTests(Folder):
    TEXT = "# Changelog\n\n## [Unreleased]\n- one\n* two\nnot a bullet\n\n## [1.0.0] - first\n- old\n"

    def test_the_bullets_under_the_unreleased_heading_are_counted_up_to_the_next_heading(self) -> None:
        self.assertEqual(CP.unreleased_entries(self.TEXT), 2)

    def test_the_count_is_the_same_with_the_line_endings_of_windows(self) -> None:
        self.assertEqual(CP.unreleased_entries(self.TEXT.replace("\n", "\r\n")), 2)

    def test_a_changelog_without_the_heading_or_empty_has_no_entries(self) -> None:
        self.assertEqual(CP.unreleased_entries("# Changelog\n\n## [1.0.0]\n- x\n"), 0)
        self.assertEqual(CP.unreleased_entries(""), 0)
        self.assertEqual(CP.unreleased_entries("## [Unreleased]\n"), 0)

    def test_the_heading_is_found_whatever_the_case_and_a_last_section_goes_to_the_end(self) -> None:
        self.assertEqual(CP.unreleased_entries("## [unreleased]\n- a\n- b\n- c\n"), 3)

    def test_the_lines_of_the_unreleased_part_are_the_stripped_bullets_only(self) -> None:
        self.assertEqual(CP.unreleased_lines(self.TEXT), {"- one", "* two"})
        self.assertEqual(CP.unreleased_lines("## [1.0.0]\n- x\n"), set())

    def test_the_versions_are_the_headings_of_the_changelog(self) -> None:
        self.assertEqual(CP.versions(self.TEXT), {"Unreleased", "1.0.0"})
        self.assertEqual(CP.versions("nothing"), set())

    def test_the_watched_folders_are_the_default_unless_the_configuration_lists_them(self) -> None:
        self.assertEqual(CP.watched_paths(self.root), CP.DEFAULT_PATHS)
        self.put("tools/ci_config.json", json.dumps({"changelog_paths": ["lib/", "bin/"]}))
        self.assertEqual(CP.watched_paths(self.root), ("lib/", "bin/"))

    def test_a_configuration_that_is_damaged_or_of_the_wrong_type_falls_back_to_the_default(self) -> None:
        self.put("tools/ci_config.json", "{broken")
        self.assertEqual(CP.watched_paths(self.root), CP.DEFAULT_PATHS)
        self.put("tools/ci_config.json", json.dumps({"changelog_paths": "lib/"}))
        self.assertEqual(CP.watched_paths(self.root), CP.DEFAULT_PATHS)
        self.put("tools/ci_config.json", json.dumps({"changelog_paths": ["lib/", 3]}))
        self.assertEqual(CP.watched_paths(self.root), CP.DEFAULT_PATHS)

    def test_the_command_without_the_base_ends_with_the_code_of_wrong_use(self) -> None:
        import contextlib
        import io

        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as raised:
            CP.main([])
        self.assertEqual(raised.exception.code, 64)


class BannerCheckTests(Folder):
    MANIFEST = {"name": "EH3D-X", "role": "service", "technologies": ["Rust", "Python"]}

    def banner(self, tag: str = "EH3D · SERVICE", badge: str = "SERVICE", line: str = "RUST · PYTHON", pills: tuple[str, ...] = ("RUST", "PYTHON"), tagline: str = "A slogan") -> str:
        parts = ['<svg>', '<text font-size="40">EH3D-X</text>', f"<text>{tag}</text>", f"<text>{badge}</text>", f"<text>{line}</text>"]
        parts += [f'<text font-size="10" letter-spacing="0.5">{p}</text>' for p in pills]
        parts += [f'<text font-size="22">{tagline}</text>', "</svg>"]
        return "\n".join(parts)

    def test_the_text_of_a_picture_comes_in_document_order_without_tags_and_unescaped(self) -> None:
        svg = '<svg><text a="1" b="2">Fish &amp; <tspan>Chips</tspan></text><text>  second </text></svg>'
        self.assertEqual(BC.texts(svg), [({"a": "1", "b": "2"}, "Fish & Chips"), ({}, "second")])

    def test_a_banner_that_says_what_the_manifest_says_has_no_problem(self) -> None:
        self.assertEqual(BC.problems_of(self.banner(), self.MANIFEST, "images/b.svg"), [])

    def test_a_banner_with_another_role_is_reported_twice_and_with_its_label(self) -> None:
        found = BC.problems_of(self.banner(tag="EH3D · TOOL", badge="TOOL"), self.MANIFEST, "images/b.svg")
        self.assertEqual(len(found), 2)
        self.assertTrue(all(line.startswith("images/b.svg: ") for line in found))

    def test_pills_that_differ_from_the_first_four_technologies_are_reported(self) -> None:
        found = BC.problems_of(self.banner(pills=("RUST",)), self.MANIFEST, "b")
        self.assertEqual(len(found), 1)
        self.assertIn("the pills show", found[0])

    def test_a_banner_without_a_tagline_is_reported(self) -> None:
        found = BC.problems_of(self.banner(tagline=""), self.MANIFEST, "b")
        self.assertEqual(found, ["b: the banner has no tagline"])

    def test_the_plain_banner_of_a_new_repository_is_checked_for_its_name_alone(self) -> None:
        plain = '<svg><text>EH3D-X</text><text>a slogan</text></svg>'
        self.assertEqual(BC.problems_of(plain, self.MANIFEST, "b"), [])
        wrong = '<svg><text>EH3D-OTHER</text></svg>'
        self.assertEqual(len(BC.problems_of(wrong, self.MANIFEST, "b")), 1)

    def test_only_the_svg_files_with_banner_in_the_name_are_banners_and_in_order(self) -> None:
        self.assertEqual(BC.banner_files(self.root), [])
        for name in ("b_BANNER.svg", "a-banner.SVG", "logo.svg", "banner.png"):
            self.put(f"images/{name}", "<svg/>")
        self.assertEqual([p.name for p in BC.banner_files(self.root)], ["a-banner.SVG", "b_BANNER.svg"])

    def test_a_repository_without_a_banner_or_with_a_manifest_that_is_not_json_has_nothing_to_report(self) -> None:
        self.assertEqual(BC.problems(self.root), [])
        self.put("images/banner.svg", "<svg><text>other</text></svg>")
        self.assertEqual(BC.problems(self.root), [])
        self.put("eh3d.project.json", "{oops")
        self.assertEqual(BC.problems(self.root), [])

    def test_a_banner_with_the_wrong_title_is_reported_by_the_check_of_the_repository(self) -> None:
        self.put("images/banner.svg", "<svg><text>other</text></svg>")
        self.put("eh3d.project.json", json.dumps(self.MANIFEST))
        found = BC.problems(self.root)
        self.assertEqual(len(found), 1)
        self.assertTrue(found[0].startswith("images/banner.svg: "))

    def test_the_command_prints_the_verdict_and_exits_with_the_right_code(self) -> None:
        import contextlib
        import io

        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            self.assertEqual(BC.main(["--repo", str(self.root)]), 0)
        self.assertIn("BANNER_CHECK=PASS", out.getvalue())
        self.put("images/banner.svg", "<svg><text>other</text></svg>")
        self.put("eh3d.project.json", json.dumps(self.MANIFEST))
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            self.assertEqual(BC.main(["--repo", str(self.root)]), 1)
        self.assertIn("BANNER_CHECK=FAIL", out.getvalue())

    def test_the_command_with_an_unknown_argument_exits_64_and_with_help_exits_0(self) -> None:
        import contextlib
        import io

        with contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(BC.main(["--nope"]), 64)
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(BC.main(["--help"]), 0)


if __name__ == "__main__":
    unittest.main()
