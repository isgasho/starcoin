address 0x1 {

/**
This module defines the Option type and its methods to represent and handle an optional value.
*/
module Option {
    use 0x1::Vector;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    /// Abstraction of a value that may or may not be present. Implemented with a vector of size
    /// zero or one because Move bytecode does not have ADTs.
    struct Option<Element> {
        vec: vector<Element>
    }
    spec struct Option {
        /// The size of vector is always less than equal to 1
        /// because it's 0 for "none" or 1 for "some".
        invariant len(vec) <= 1;
    }

    const EOPTION_ALREADY_FILLED: u64 = 0;

    /// Return an empty `Option`
    public fun none<Element>(): Option<Element> {
        Option { vec: Vector::empty() }
    }
    spec fun none {
        pragma opaque = true;
        aborts_if false;
        ensures result == spec_none<Element>();
    }
    spec module {
        define spec_none<Element>(): Option<Element> {
            Option{ vec: empty_vector() }
        }
    }

    /// Return an `Option` containing `e`
    public fun some<Element>(e: Element): Option<Element> {
        Option { vec: Vector::singleton(e) }
    }
    spec fun some {
        //pragma opaque = true;
        aborts_if false;
        ensures result == spec_some(e);
    }
    spec module {
        define spec_some<Element>(e: Element): Option<Element> {
            Option{ vec: Vector::spec_singleton(e) }
        }
    }

    /// Return true if `t` does not hold a value
    public fun is_none<Element>(t: &Option<Element>): bool {
        Vector::is_empty(&t.vec)
    }
    spec fun is_none {
        pragma opaque = true;
        aborts_if false;
        ensures result == spec_is_none(t);
    }
    spec module {
        define spec_is_none<Element>(t: Option<Element>): bool {
            len(t.vec) == 0
        }
    }

    /// Return true if `t` holds a value
    public fun is_some<Element>(t: &Option<Element>): bool {
        !Vector::is_empty(&t.vec)
    }
    spec fun is_some {
        pragma opaque = true;
        aborts_if false;
        ensures result == spec_is_some(t);
    }
    spec module {
        define spec_is_some<Element>(t: Option<Element>): bool {
            !spec_is_none(t)
        }
    }

    /// Return true if the value in `t` is equal to `e_ref`
    /// Always returns `false` if `t` does not hold a value
    public fun contains<Element>(t: &Option<Element>, e_ref: &Element): bool {
        Vector::contains(&t.vec, e_ref)
    }
    spec fun contains {
        pragma opaque = true;
        aborts_if false;
        ensures result == (spec_is_some(t) && spec_get(t) == e_ref);
    }
    spec module {
        define spec_contains<Element>(t: Option<Element>, e: Element): bool {
            spec_is_some(t) && spec_get(t) == e
        }
    }

    /// Return an immutable reference to the value inside `t`
    /// Aborts if `t` does not hold a value
    public fun borrow<Element>(t: &Option<Element>): &Element {
        Vector::borrow(&t.vec, 0)
    }
    spec fun borrow {
        pragma opaque = true;
        aborts_if spec_is_none(t);
        ensures result == spec_get(t);
    }
    spec module {
        define spec_get<Element>(t: Option<Element>): Element {
            t.vec[0]
        }
    }

    /// Return a reference to the value inside `t` if it holds one
    /// Return `default_ref` if `t` does not hold a value
    public fun borrow_with_default<Element>(t: &Option<Element>, default_ref: &Element): &Element {
        let vec_ref = &t.vec;
        if (Vector::is_empty(vec_ref)) default_ref
        else Vector::borrow(vec_ref, 0)
    }
    spec fun borrow_with_default {
        pragma opaque = true;
        aborts_if false;
        ensures spec_is_none(t) ==> result == default_ref;
        ensures spec_is_some(t) ==> result == spec_get(t);
    }

    /// Return the value inside `t` if it holds one
    /// Return `default` if `t` does not hold a value
    public fun get_with_default<Element: copyable>(t: &Option<Element>, default: Element): Element {
        let vec_ref = &t.vec;
        if (Vector::is_empty(vec_ref)) default
        else *Vector::borrow(vec_ref, 0)
    }
    spec fun get_with_default {
        pragma opaque = true;
        aborts_if false;
        ensures spec_is_none(t) ==> result == default;
        ensures spec_is_some(t) ==> result == spec_get(t);
    }

    /// Convert the none option `t` to a some option by adding `e`.
    /// Aborts if `t` already holds a value
    public fun fill<Element>(t: &mut Option<Element>, e: Element) {
        let vec_ref = &mut t.vec;
        if (Vector::is_empty(vec_ref)) Vector::push_back(vec_ref, e)
        else abort EOPTION_ALREADY_FILLED
    }
    spec fun fill {
        pragma opaque = true;
        aborts_if spec_is_some(t);
        ensures spec_is_some(t);
        ensures spec_get(t) == e;
    }

    /// Convert a `some` option to a `none` by removing and returning the value stored inside `t`
    /// Aborts if `t` does not hold a value
    public fun extract<Element>(t: &mut Option<Element>): Element {
        Vector::pop_back(&mut t.vec)
    }
    spec fun extract {
        pragma opaque = true;
        aborts_if spec_is_none(t);
        ensures result == spec_get(old(t));
        ensures spec_is_none(t);
    }

    /// Return a mutable reference to the value inside `t`
    /// Aborts if `t` does not hold a value
    public fun borrow_mut<Element>(t: &mut Option<Element>): &mut Element {
        Vector::borrow_mut(&mut t.vec, 0)
    }
    spec fun borrow_mut {
        pragma opaque = true;
        aborts_if spec_is_none(t);
        ensures result == spec_get(t);
    }

    /// Swap the old value inside `t` with `e` and return the old value
    /// Aborts if `t` does not hold a value
    public fun swap<Element>(t: &mut Option<Element>, e: Element): Element {
        let vec_ref = &mut t.vec;
        let old_value = Vector::pop_back(vec_ref);
        Vector::push_back(vec_ref, e);
        old_value
    }
    spec fun swap {
        pragma opaque = true;
        aborts_if spec_is_none(t);
        ensures result == spec_get(old(t));
        ensures spec_is_some(t);
        ensures spec_get(t) == e;
    }

    /// Destroys `t.` If `t` holds a value, return it. Returns `default` otherwise
    public fun destroy_with_default<Element: copyable>(t: Option<Element>, default: Element): Element {
        let Option { vec } = t;
        if (Vector::is_empty(&mut vec)) default
        else Vector::pop_back(&mut vec)
    }
    spec fun destroy_with_default {
        pragma opaque = true;
        aborts_if false;
        ensures spec_is_none(old(t)) ==> result == default;
        ensures spec_is_some(old(t)) ==> result == spec_get(old(t));
    }

    /// Unpack `t` and return its contents
    /// Aborts if `t` does not hold a value
    public fun destroy_some<Element>(t: Option<Element>): Element {
        let Option { vec } = t;
        let elem = Vector::pop_back(&mut vec);
        Vector::destroy_empty(vec);
        elem
    }
    spec fun destroy_some {
        pragma opaque = true;
        aborts_if spec_is_none(t);
        ensures result == spec_get(old(t));
    }


    /// Unpack `t`
    /// Aborts if `t` holds a value
    public fun destroy_none<Element>(t: Option<Element>) {
        let Option { vec } = t;
        Vector::destroy_empty(vec)
    }
    spec fun destroy_none {
        pragma opaque = true;
        aborts_if spec_is_some(t);
    }
}

}
