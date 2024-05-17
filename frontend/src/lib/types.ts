/**
 * This type allows to verify at compile time that `TPartial` contains only properties from type
 * `T`, all optional. It's similar to recursively applying {@link Partial} while ensuring that no
 * extra keys are present.
 */
export type StrictPartial<TPartial, T extends TPartial> = TPartial extends object
    ? {
          [P in keyof TPartial]?: P extends keyof TPartial
              ? StrictPartial<TPartial[P], T[P]>
              : never;
      }
    : TPartial;
