from abc import ABC, abstractmethod
from typing import Literal


def add_repr(cls):
    cls.__repr__ = (
        lambda self: f"{self.__class__.__name__}({', '.join(f'{k}={v!r}' for k, v in self.__dict__.items())})"
    )
    return cls


# Input Cond
@add_repr
class Always:
    pass


@add_repr
class MatchesKey:
    __match_args__ = ("key",)

    def __init__(self, key: str):
        self.key = key


@add_repr
class MatchesValue:
    __match_args__ = ("value",)

    def __init__(self, value):
        self.value = value


@add_repr
class Or:
    __match_args__ = ("conds",)

    def __init__(self, conds: list["InputCond"]):
        self.conds = conds


# Key Rule
@add_repr
class Any:
    pass


@add_repr
class Never:
    pass


@add_repr
class Identity:
    pass


@add_repr
class Fixed:
    __match_args__ = ("key",)

    def __init__(self, key: str):
        self.key = key


@add_repr
class IdWithPrefix:
    __match_args__ = ("prefix",)

    def __init__(self, prefix: str):
        self.prefix = prefix


KeyRule = Literal["Any"] | Never | Identity | Fixed | IdWithPrefix

InputCond = Always | MatchesKey | MatchesValue | Or
"""
Usage example:
    match input_cond:
        case Always():
            pass
        case MatchesKey(key):
            pass
        case MatchesValue(value):
            pass
        case Or(conds):
            pass
"""


@add_repr
class OutputSchema:
    def __init__(self):
        self.fixed_keys = set[str]()
        self.dynamic_keys: list[tuple[KeyRule, InputCond]] = []

    def add_fixed(self, key: str):
        self.fixed_keys.add(key)
        return self

    def add_rule_for_every_input(self, key: KeyRule, cond: InputCond):
        self.dynamic_keys.append((key, cond))
        return self

    def carry_all(self):
        self.add_rule_for_every_input(Identity(), Always())
        return self


class WorkflowVerifier(ABC):
    @abstractmethod
    def add_node(
        self, name: str, required_inputs: list[str], output_schema: OutputSchema
    ):
        pass

    @abstractmethod
    def add_edge(
        self, src: str, dst: str, additional_transition_condition: list[InputCond]
    ):
        pass

    @abstractmethod
    def set_start_node(self, node_name: str):
        pass

    @abstractmethod
    def is_reachable(self, node_name: str) -> bool:
        pass

    @abstractmethod
    def is_eventually_reached(self, target_nodes: list[str]) -> bool:
        pass


class DummyVerifier(WorkflowVerifier):
    def __init__(self) -> None:
        super().__init__()
        self.nodes = dict[str, tuple[list[str], OutputSchema]]()
        self.edges = dict[str, dict[str, list[InputCond]]]()
        self.start_node: str | None = None

    def add_node(
        self, name: str, required_inputs: list[str], output_schema: OutputSchema
    ):
        self.nodes[name] = (required_inputs, output_schema)
        self.edges[name] = dict[str, list[InputCond]]()

    def add_edge(
        self, src: str, dst: str, additional_transition_condition: list[InputCond]
    ):
        if src not in self.nodes:
            raise ValueError(f"Node {src} does not exist")
        if dst not in self.nodes:
            raise ValueError(f"Node {dst} does not exist")
        self.edges[src][dst] = additional_transition_condition

    def set_start_node(self, node_name: str):
        self.start_node = node_name

    def is_reachable(self, node_name: str) -> bool:
        return False

    def is_eventually_reached(self, target_nodes: list[str]) -> bool:
        return False

    def print_graph(self):
        print("Nodes:")
        for name, (required_inputs, output_schema) in self.nodes.items():
            print(f"  Node {name}:")
            print(f"    Required Inputs: {required_inputs}")
            print(f"    Output Schema: {output_schema}")
        print("Edges:")
        for src, dsts in self.edges.items():
            print(f"  Node {src} has edges to:")
            for dst, conditions in dsts.items():
                print(f"    Node {dst} with conditions {conditions}")
