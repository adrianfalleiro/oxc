//! Detect decorators and auto-accessors in class syntax.

use oxc_ast::ast::{Class, ClassBody, ClassElement};

/// Whether a class contains runtime auto-accessors.
pub fn class_body_has_auto_accessors(body: &ClassBody<'_>) -> bool {
    body.body.iter().any(is_auto_accessor)
}

/// Whether a class contains runtime static auto-accessors.
pub fn class_body_has_static_auto_accessor(body: &ClassBody<'_>) -> bool {
    body.body.iter().any(|element| is_auto_accessor(element) && element.r#static())
}

/// Whether a class element is a runtime auto-accessor.
pub fn is_auto_accessor(element: &ClassElement<'_>) -> bool {
    matches!(element, ClassElement::AccessorProperty(prop) if !prop.r#type.is_abstract())
}

/// Whether the class or one of its runtime elements is decorated.
///
/// Decorators on elements that TypeScript erases (`declare` fields, abstract members and method
/// overloads) never run, so they do not make the class decorated.
pub fn class_has_decorators(class: &Class<'_>) -> bool {
    !class.declare
        && (!class.decorators.is_empty()
            || class.body.body.iter().any(|element| match element {
                ClassElement::MethodDefinition(method) => {
                    !method.decorators.is_empty()
                        && !method.r#type.is_abstract()
                        && method.value.body.is_some()
                }
                ClassElement::PropertyDefinition(prop) => {
                    !prop.decorators.is_empty() && !prop.declare && !prop.r#type.is_abstract()
                }
                ClassElement::AccessorProperty(prop) => {
                    !prop.decorators.is_empty() && !prop.r#type.is_abstract()
                }
                ClassElement::StaticBlock(_) | ClassElement::TSIndexSignature(_) => false,
            }))
}

/// Whether legacy decorators lower this class by decorating the class itself, which reassigns
/// the class binding to the value returned by `_decorate`.
///
/// That is the case for a runtime class declaration whose class or constructor parameters are
/// decorated.
pub fn legacy_decorators_reassign_class_binding(class: &Class<'_>) -> bool {
    if class.is_expression() || class.declare {
        return false;
    }
    !class.decorators.is_empty()
        || class.body.body.iter().any(|element| {
            matches!(element, ClassElement::MethodDefinition(method)
                if method.kind.is_constructor()
                    && method.value.params.items.iter().any(|param| !param.decorators.is_empty()))
        })
}
