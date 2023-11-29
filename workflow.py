from abc import ABC, abstractmethod

# Input Cond
class Always:
    pass
class MatchesKey:
    def __init__(self, key: str):
        self.key = key
class MatchesValue:
    def __init__(self, value):
        self.value = value
class Or:
    def __init__(self, conds: list['InputCond']):
        self.conds = conds

# Key Rule
class Any:
    pass
class Never:
    pass
class Identity:
    pass
class Fixed:
    def __init__(self, key: str):
        self.key = key
class IdWithPrefix:
    def __init__(self, prefix: str):
        self.prefix = prefix

type KeyRule = Any | Never | Identity | Fixed | IdWithPrefix

type InputCond = Always | MatchesKey | MatchesValue | Or 
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

class OutputSchema:
    def __init__(self):
        raise NotImplementedError # TODO
    
    def add_fixed(self, key: str):
        raise NotImplementedError # TODO
    
    def add_rule_for_every_input(self, key: KeyRule, cond: InputCond):
        raise NotImplementedError # TODO
    
    def carry_all(self):
        self.add_rule_for_every_input(Identity, Always)

class Workflow(ABC):
    @abstractmethod
    def add_node(self, name: str, required_inputs: list[str], output_schema: OutputSchema):
        pass

    @abstractmethod
    def add_edge(self, src: str, dst: str, additional_transition_condition: list[InputCond]):
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