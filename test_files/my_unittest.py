import unittest
from unittest import SkipTest


def my_skip():
    raise unittest.SkipTest("reason")


def my_skip2():
    raise SkipTest()


class MyClass:
    def my__skip():
        raise SkipTest()
