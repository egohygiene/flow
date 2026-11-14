"""Adversarial checks for the PTY observer's acceptance boundary."""
import unittest
from check_root_help_pty import compare


class RootHelpObserver(unittest.TestCase):
    plain = (0, b"tool purpose\nCommands: help\n", b"")
    mark = b"STATIC ART\n"

    def test_color_is_allowed_only_on_selected_capable_paths(self):
        colored = (0, b"\x1b[1mtool purpose\x1b[0m\n" + self.mark + b"Commands: help\n", b"")
        compare(colored, self.plain, self.mark, True, allow_color=True)
        with self.assertRaises(ValueError):
            compare(colored, self.plain, self.mark, True)

    def test_no_art_cannot_satisfy_decorated_case(self):
        with self.assertRaises(ValueError):
            compare(self.plain, self.plain, self.mark, True)

    def test_semantic_changes_cannot_hide_behind_art(self):
        with self.assertRaises(ValueError):
            compare((0, self.mark + b"wrong help\n", b""), self.plain, self.mark, True)

    def test_repeated_art_fails(self):
        with self.assertRaises(ValueError):
            compare((0, self.mark * 2 + self.plain[1], b""), self.plain, self.mark, True)

    def test_panic_or_other_exit_is_not_success(self):
        with self.assertRaises(ValueError):
            compare((101, self.plain[1], b""), self.plain, self.mark, False)

    def test_extra_stderr_fails(self):
        with self.assertRaises(ValueError):
            compare((0, self.plain[1], b"unexpected"), self.plain, self.mark, False)

    def test_cursor_control_is_never_accepted_as_color(self):
        with self.assertRaises(ValueError):
            compare((0, b"\x1b[2J" + self.plain[1], b""), self.plain, self.mark, False, allow_color=True)

    def test_plain_path_cannot_emit_art(self):
        with self.assertRaises(ValueError):
            compare((0, self.mark + self.plain[1], b""), self.plain, self.mark, False)


if __name__ == "__main__":
    unittest.main()
