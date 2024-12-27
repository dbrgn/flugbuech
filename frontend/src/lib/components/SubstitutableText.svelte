<!--
    @component Render text where a placeholder can be replaced with a slot.

    Example with a placeholder being replaced with <br>:

        <SubstitutableText text="Hello<1/>World">
            <br slot="1" />
        </SubstitutableText>

    Example with a placeholder wrapped in an <a> tag:

        <SubstitutableText text="Hello <1>World</1>">
            <a slot="1" href="https://example.com/" target="_blank" let:text>{text}</a>
        </SubstitutableText>

    Licensing: This component is originally based on `src/app/ui/SubstitutableText.svelte` as part
    of Threema Desktop (https://github.com/threema-ch/threema-desktop/), which is released under the
    AGPLv3 license.
-->
<script lang="ts">
  import {assertUnreachable, unreachable, unwrap} from '$lib/assert';

  export let text: string | undefined;

  // For now there are no instances of needing more than 3 different tags in a text. We can add more
  // if needed.
  const ALLOWED_TAGS = ['1', '2', '3'] as const;
  type AllowedTag = (typeof ALLOWED_TAGS)[number];

  function isAllowedTag(tag: string | undefined): tag is AllowedTag {
    if (tag === undefined) {
      return false;
    }
    return (ALLOWED_TAGS as readonly string[]).includes(tag);
  }

  const ALLOWED_TAGS_CHAR_SET = `[${ALLOWED_TAGS.join('')}]`;
  const SELF_CLOSING_TAG_PATTERN = `<(?<selfClosingTag>${ALLOWED_TAGS_CHAR_SET}) ?/>` as const;
  const TAG_PATTERN = `<(?<tag>${ALLOWED_TAGS_CHAR_SET})>(?<text>.*?)</\\k<tag>>` as const;
  const PLAIN_TEXT_PATTERN = `(?<plain>(?:.+?(?=<${ALLOWED_TAGS_CHAR_SET}(?: ?/)?>)|.+$))` as const;

  const TAG_SPLITTER_REGEX = new RegExp(
    [SELF_CLOSING_TAG_PATTERN, TAG_PATTERN, PLAIN_TEXT_PATTERN].join('|'),
    'gum',
  );

  type Fragment =
    | {readonly type: 'plain'; readonly text: string}
    | {readonly type: 'tag'; readonly tag: AllowedTag; readonly text: string}
    | {readonly type: 'selfClosingTag'; readonly tag: AllowedTag; readonly text: undefined};

  function warnMissingSlot(tag: AllowedTag): void {
    console.warn(
      `Text "${text}" expects a child slot with \`name="${tag}"\` but it has not been provided.`,
    );
  }

  function warnUnusedSlots(newFragments: Fragment[]): void {
    const expectedTags = new Set(
      newFragments
        .map((fragment) => (fragment.type === 'plain' ? '' : fragment.tag))
        .filter(isAllowedTag),
    );
    for (const tag of ALLOWED_TAGS) {
      if ($$slots[tag] && !expectedTags.has(tag)) {
        console.warn(`Unused child slot with \`name="${tag}"\` for text "${text}".`);
      }
    }
  }

  $: fragments =
    text === undefined
      ? []
      : [...text.matchAll(TAG_SPLITTER_REGEX)].map<Fragment>((match) => {
          const matchedText = match[0];
          const {groups} = match;

          if (groups === undefined) {
            return assertUnreachable('TAG_SPLITTER_REGEX should have returned a matched group');
          }

          if (groups.plain !== undefined) {
            return {
              type: 'plain',
              text: groups.plain,
            };
          }

          if (isAllowedTag(groups.tag)) {
            const tagText = unwrap(groups.text);
            if ($$slots[groups.tag]) {
              return {
                type: 'tag',
                tag: groups.tag,
                text: tagText,
              };
            }
            warnMissingSlot(groups.tag);
            return {
              type: 'plain',
              text: import.meta.env.DEBUG ? matchedText : tagText,
            };
          }

          if (isAllowedTag(groups.selfClosingTag)) {
            if ($$slots[groups.selfClosingTag]) {
              return {
                type: 'selfClosingTag',
                tag: groups.selfClosingTag,
              };
            }
            warnMissingSlot(groups.selfClosingTag);
            return {
              type: 'plain',
              text: import.meta.env.DEBUG ? matchedText : '',
            };
          }

          return assertUnreachable(`Unexpected matching by TAG_SPLITTER_REGEX on '${matchedText}'`);
        });

  $: warnUnusedSlots(fragments);
</script>

{#each fragments as fragment (fragment)}
  {#if fragment.type === 'plain'}
    {fragment.text}
  {:else if fragment.type === 'tag' || fragment.type === 'selfClosingTag'}
    {#if fragment.tag === '1'}
      <slot name="1" text={fragment.text} />
    {:else if fragment.tag === '2'}
      <slot name="2" text={fragment.text} />
    {:else if fragment.tag === '3'}
      <slot name="3" text={fragment.text} />
    {:else}
      {unreachable(fragment.tag)}
    {/if}
  {:else}
    {unreachable(fragment)}
  {/if}
{/each}
