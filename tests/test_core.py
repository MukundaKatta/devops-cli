"""Tests for DevopsCli."""
from src.core import DevopsCli
def test_init(): assert DevopsCli().get_stats()["ops"] == 0
def test_op(): c = DevopsCli(); c.detect(x=1); assert c.get_stats()["ops"] == 1
def test_multi(): c = DevopsCli(); [c.detect() for _ in range(5)]; assert c.get_stats()["ops"] == 5
def test_reset(): c = DevopsCli(); c.detect(); c.reset(); assert c.get_stats()["ops"] == 0
def test_service_name(): c = DevopsCli(); r = c.detect(); assert r["service"] == "devops-cli"
