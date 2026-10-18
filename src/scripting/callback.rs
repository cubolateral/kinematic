use mlua::{Function, Result};

// Lua 5.4 retains declaration lines even in stripped dumps; they do not identify callback logic.
pub(super) fn key(function: &Function) -> Result<String> {
    let bytes = function.dump(true);
    let result = (|| {
        if bytes.get(..6)? != b"\x1bLua\x54\0" || bytes.get(12..15)? != [4, 8, 8] {
            return None;
        }

        let mut remaining = bytes.get(32..)?;
        let mut output = Vec::new();

        prototype(&mut remaining, &mut output, true)?;

        remaining.is_empty().then(|| format!("{output:?}"))
    })();

    result.ok_or_else(|| mlua::Error::runtime("Unsupported Lua 5.4 callback bytecode."))
}

fn take<'a>(input: &mut &'a [u8], count: usize) -> Option<&'a [u8]> {
    let (head, tail) = input.split_at_checked(count)?;

    *input = tail;

    Some(head)
}

fn count(input: &mut &[u8], output: &mut Vec<u8>) -> Option<usize> {
    let mut value = 0usize;

    loop {
        let byte = take(input, 1)?[0];

        output.push(byte);

        value = value.checked_mul(128)?.checked_add((byte & 127) as usize)?;

        if byte & 128 != 0 {
            return Some(value);
        }
    }
}

fn string(input: &mut &[u8], output: &mut Vec<u8>) -> Option<()> {
    let size = count(input, output)?;

    output.extend_from_slice(take(input, size.saturating_sub(1))?);

    Some(())
}

fn prototype(input: &mut &[u8], output: &mut Vec<u8>, root: bool) -> Option<()> {
    let mut discarded = Vec::new();

    string(input, &mut discarded)?;
    count(input, &mut discarded)?;
    count(input, &mut discarded)?;
    output.extend_from_slice(take(input, 3)?);

    let instructions = count(input, output)?;

    output.extend_from_slice(take(input, instructions.checked_mul(4)?)?);

    let constants = count(input, output)?;

    for _ in 0..constants {
        let tag = take(input, 1)?[0];

        output.push(tag);

        match tag {
            0 | 1 | 17 => (),
            3 | 19 => output.extend_from_slice(take(input, 8)?),
            4 | 20 => string(input, output)?,
            _ => return None,
        }
    }

    let upvalues = count(input, output)?;
    let descriptors = take(input, upvalues.checked_mul(3)?)?;

    // A pure update can only capture _ENV; its position in the parent scope is irrelevant.
    if !root {
        output.extend_from_slice(descriptors);
    }

    let children = count(input, output)?;

    for _ in 0..children {
        prototype(input, output, false)?;
    }

    for _ in 0..4 {
        if count(input, &mut discarded)? != 0 {
            return None;
        }
    }

    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_keys_ignore_layout_and_unrelated_literals_but_keep_nested_logic() {
        let lua = mlua::Lua::new();
        let a = lua.load("local color='red'; return function(state) local function double(x) return x*2 end; state.total=double(state.total)+math.sin(1) end").eval::<Function>().unwrap();
        let b = lua.load("-- Comment.\nlocal color='blue'\nreturn function(state)\nlocal function double(x) return x * 2 end\nstate.total = double(state.total) + math.sin(1)\nend").eval::<Function>().unwrap();
        let changed = lua.load("return function(state) local function double(x) return x*3 end; state.total=double(state.total)+math.sin(1) end").eval::<Function>().unwrap();

        assert_eq!(key(&a).unwrap(), key(&b).unwrap());
        assert_ne!(key(&a).unwrap(), key(&changed).unwrap());
    }
}
