# nkts command specification

All assumed by the actual play & the syntax. No guarantee to be accurate.

In nkts script, every command is in the following format:
```
$CMD_NAME,arg1,arg2,arg3,...
```
and is written in this document as the following format (as for convenience):
```
CmdName(A1, A2, A3, ...)
```

## `Draw`

### Syntax
```
Draw(D)
```

### Behaviour

Flush all the drawing commands and fade into the newly-drawn buffer.

### Parameters
- `D` : duration (in milliseconds)

## `Face`

### `Face()`

#### Syntax
```
Face()
```

#### Behaviour

Clear the face layer.

#### Paramters

None.

### `Face(P, 'm', PL1, PL2...)`

#### Syntax
```
Face(P, 'm', PL1, PL2...)
```

#### Behaviour

Display the image on the face layer.

#### Paramters
- `P` : path to the image
- `PL1, PL2, ...` : choices of pict-layer

## `FaceAuto`

### Syntax
```
FaceAuto(F)
```

### Behaviour

Set the flag to enable auto-face display.

### Paramters
- `F` : the flag value

## `FaceAnime`

### Syntax
```
FaceAnime(F)
```

### Behaviour

Set the flag to enable face animation.

### Paramters
- `F` : the flag value

## `AChr`

### 02: `AChr(02, L, X, Y, D)`

Bounce. Move to (X, Y) and move back to the original position.

**TODO: move behaviour**

#### Parameters
- `L` : layer to fade
- `X, Y` : position
- `D` : duration (in milliseconds)

### 11: `AChr(11, L, N1, N2, N3, D)`

Blink.

#### Parameters
- `L` : layer to fade
- `N1, N2, N3` : unknown parameters
- `D` : duration (in milliseconds)

### 60: `AChr(60, L, P, D)`

Overlay fade-in?

```glsl
uniform sampler2D tex;
uniform sampler2D ov;
uniform float     fade_amount;

void frag() {
    // assume that overlay image is monochrome;
    // i.e. green channel is only used.

    f_color.rgb = texture(tex, uv).rgb;
    f_color.a = texture(tex, uv).a
                 * max(1.0, fade_amount + texture(ov, uv));
}
```

#### Parameters
- `L` : layer to fade
- `P` : path to the overlay image
- `D` : duration (in milliseconds)

### 128: `AChr(128, L, X, Y, D, N1)`

Move the layer.

#### Parameters
- `X, Y` : position
- `D` : duration (in milliseconds)
- `N1` : Unknown parameter. (probably easing option?)

### 150: `AChr(150, L, D)`

Fade the specified layer out.

#### Parameters
- `L` : layer to fade
- `D` : duration (in milliseconds)

### 151: `AChr(151, L, D)`

Fade the specified layer in.

#### Parameters
- `L` : layer to fade
- `D` : duration (in milliseconds)
