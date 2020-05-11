# Commands

All assumed by the actual play & the syntax. No guarantee to be accurate.

In nkts script, every command is in the following format:
```
$CMD_NAME,arg1,arg2,arg3,...
```
and is written in this document as the following format (as for convenience):
```
CmdName(A1, A2, A3, ...)
```

## `AChr`

### 01 `BOUNCE_X`: `AChr(02, L, N, DX, D)`

The bounce effect. Move by (DX, 0) and move back to the original position.
The total duration of the animation is `D * N`.

**TODO: move behaviour**

#### Parameters
- `L` : layer to fade
- `N` : total bounces
- `DX` : position
- `D` : duration (in milliseconds)


### 02 `BOUNCE_Y`: `AChr(02, L, N, DY, D)`

The bounce effect. Move by (0, DY) and move back to the original position.
The total duration of the animation is `D * N`.

**TODO: move behaviour**

#### Parameters
- `L` : layer to fade
- `N` : total bounces
- `DY` : position
- `D` : duration (in milliseconds)

### 06 `_`: `AChr(06, N1, N2, N3, N4)`

**TODO:**

#### Parameters
- `N1, N2, N3, N4` : unknown parameters

### 11 `BLINK`: `AChr(11, L, N1, N2, N3, D)`

Blinks the layer.

#### Parameters
- `L` : layer to fade
- `N1, N2, N3` : unknown parameters
- `D` : duration (in milliseconds)

### 20 `_`: `AChr(20, L, N1, N2, N3, N4, N5)`

**TODO:**

#### Parameters
- `L` : layer
- `N1, N2, N3, N4, N5` : unknown parameters

### 30 `_`: `AChr(30, N1, N2, N3, N4, N5)`

**TODO:**

#### Parameters
- `N1, N2, N3, N4, N5` : unknown parameters

### 60 `OVL_FADE_IN`: `AChr(60, L, P, D)`

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

### 61 `OVL_FADE_OUT`: `AChr(61, L, P, D)`

Overlay fade-out?

**TODO: not tested nor checked**

#### Parameters
- `L` : layer to fade
- `P` : path to the overlay image
- `D` : duration (in milliseconds)

### 128 `MOVE_TO`: `AChr(128, L, X, Y, D, N1)`

Moves the layer.

#### Parameters
- `X, Y` : position
- `D` : duration (in milliseconds)
- `N1` : Unknown parameter. (probably easing option?)

### 150 `FADE_OUT`: `AChr(150, L, D)`

Fades the specified layer out.

#### Parameters
- `L` : layer to fade
- `D` : duration (in milliseconds)

### 151 `FADE_IN`: `AChr(151, L, D)`

Fades the specified layer in.

#### Parameters
- `L` : layer to fade
- `D` : duration (in milliseconds)

## `Draw`

### Syntax
```
Draw(D)
```

### Behaviour

Flushes all the drawing commands and fade into the newly-drawn buffer.

### Parameters
- `D` : duration (in milliseconds)

## `Face`

### `Face()`

#### Syntax
```
Face()
```

#### Behaviour

Clears the face layer.

#### Paramters

None.

### `Face(P, 'm', PL1, PL2...)`

#### Syntax
```
Face(P, 'm', PL1, PL2...)
```

#### Behaviour

Displays the image on the face layer.

#### Paramters
- `P` : path to the image
- `PL1, PL2, ...` : choices of pict-layer

## `FaceAuto`

### Syntax
```
FaceAuto(F)
```

### Behaviour

Sets the flag to enable auto-face display.

### Paramters
- `F` : the flag value

## `FaceAnime`

### Syntax
```
FaceAnime(F)
```

### Behaviour

Sets the flag to enable face animation.

### Paramters
- `F` : the flag value

## `LChr`

### `LChr(L)`

#### Syntax
```
LChr(L)
```

#### Behaviour

Clear the layer.

#### Paramters

- `L`: the layer number

### `LChr(L, P, X, Y, E)`

#### Syntax
```
LChr(L, P, X, Y, E)
```

#### Behaviour

Displays the specified image on the specified layer at (X, Y).

#### Paramters
- `P` : path to the image
- `X, Y` : position
- `E` : unknown; probably entry number of the image

### `LDelay(L, 'T', D)`

#### Syntax
```
LDelay(L, 'T', D)`
```

#### Behaviour

Delay the animation queue of the layer. 

(If L = 0, all events are delayed?)

#### Paramters
- `L` : layer
- `D` : duration

## `LMont`

### `LMont(L, P, 'm', PL1, PL2...)`

#### Syntax
```
LMont(L, P, X, Y, N1, 'm', PL1, PL2...)
```

#### Behaviour

Displays the combined image on the specified layer at (X, Y).
The combination of pict-layer is given by `PL1, PL2, ...`.

#### Paramters
- `L` : layer to display
- `P` : path to the image
- `X, Y` : position
- `N1` : unknown
- `PL1, PL2, ...` : choices of pict-layer
