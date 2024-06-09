import unittest
import unittest as ut
from unittest import TestCase, skip
from unittest import skip as sk


def func(val: int) -> int:
    return val + 100


class AstBuildingTest(TestCase):
    """Test for AST building."""

    def test_1(self):
        """the simple test"""

        a = 1
        b = 2
        self.assertEqual(a + b, 3)

    def test_2(self):
        """関数呼び出しを行うテスト"""

        self.assertEqual(func(200), 300)

    @unittest.skip
    def test_skip(self):
        """test with skip"""
        self.fail("unexpected ignored or disabled skip decorator for method...")

    def test_3(self):
        """the simple test after skipped test"""

        self.assertTrue(True)


@unittest.skip
class SkipTest(unittest.TestCase):
    def test_may_be_skipping(self):
        self.fail("unexpected ignored or disabled skip decorator for class...")


class InnerClass:
    class InnerTestClassA(unittest.TestCase):
        def test_a(self):
            self.assertTrue(True)

        def test_b(self):
            self.assertTrue(True)

        class InnerTestClassAA(unittest.TestCase):
            def test_a(self):
                self.assertTrue(True)

            def test_b(self):
                self.assertTrue(True)

            @skip
            class InnerSkipTestAAA(TestCase):
                def test_skip(self):
                    self.assertTrue(True)

                class InnerSkipTestAAA(TestCase):
                    def test_skip(self):
                        self.assertTrue(True)

            class InnerTestAAB(TestCase):
                @skip
                def test_skip(self):
                    self.assertTrue(True)

    class InnerTestClassB(TestCase):
        def test_a(self):
            self.assertTrue(True)

        def test_b(self):
            self.assertTrue(True)

        class InnerTestClassBB(TestCase):
            def test_a(self):
                self.assertTrue(True)

            def test_b(self):
                self.assertTrue(True)

        class InnerTestClassBBB(TestCase):
            def test_a(self):
                self.assertTrue(True)

            def test_b(self):
                self.assertTrue(True)
