"""In-memory observers: a crash or duplicate inventory cannot satisfy proof."""
import unittest

from check_package_candidate import (asset_inventory, exact_lock_inventory,
    exact_verifier_refusal)


class PackageCandidateObserverTests(unittest.TestCase):
    reason = "package asset changed"

    def test_specific_python_value_error_is_accepted(self):
        exact_verifier_refusal(1,b"",b"Traceback (most recent call last):\nValueError: package asset changed\n",self.reason)

    def test_panic_101_cannot_count_as_the_expected_refusal(self):
        with self.assertRaises(ValueError):
            exact_verifier_refusal(101,b"",b"ValueError: package asset changed\n",self.reason)

    def test_expected_text_before_an_unrelated_final_error_is_refused(self):
        with self.assertRaises(ValueError):
            exact_verifier_refusal(1,b"",b"ValueError: package asset changed\nRuntimeError: unrelated crash\n",self.reason)

    def test_final_line_must_be_exact_and_newline_terminated(self):
        for stderr in (b"ValueError: package asset changed",b"prefix ValueError: package asset changed\n",b"ValueError: package asset changed\n\n"):
            with self.subTest(stderr=stderr),self.assertRaises(ValueError):
                exact_verifier_refusal(1,b"",stderr,self.reason)

    def test_refused_verifier_cannot_also_emit_a_result(self):
        with self.assertRaises(ValueError):
            exact_verifier_refusal(1,b'{"qualified":true}',b"ValueError: package asset changed\n",self.reason)

    def test_assets_require_eight_distinct_names_before_projection(self):
        good=[{"name":"asset-"+str(i)} for i in range(8)]
        self.assertEqual(len(asset_inventory(good)),8)
        duplicate=[*good[:-1],good[0]]
        for entries in (duplicate,good[:-1],[*good,{"name":"extra"}]):
            with self.subTest(entries=entries),self.assertRaises(ValueError):asset_inventory(entries)

    def test_spdx_requires_the_exact_unique_lock_inventory(self):
        packages=[{"name":"a","versionInfo":"1"},{"name":"b","versionInfo":"2"}]
        locked=[{"name":"a","version":"1"},{"name":"b","version":"2"}]
        exact_lock_inventory(packages,locked)
        for actual,expected in (([*packages,packages[0]],locked),(packages,[*locked,locked[0]]),(packages[:1],locked)):
            with self.subTest(actual=actual,expected=expected),self.assertRaises(ValueError):
                exact_lock_inventory(actual,expected)


if __name__ == "__main__":unittest.main()
