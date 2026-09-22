import { useState } from 'react';
import type { Language } from '../../journal/types';
import { tr } from '../../journal/copy';

type ProfilePickerProps = {
  activeProfile: string | null;
  language?: Language;
  profiles: string[];
  onSelect: (profileId: string) => void;
};

export function ProfilePicker({ activeProfile, language = 'eng', profiles, onSelect }: ProfilePickerProps) {
  const text = {
    eyebrow: tr(language, 'profilesEyebrow'),
    title: tr(language, 'profileTitle'),
    help: tr(language, 'profileHelp'),
    saved: tr(language, 'savedProfiles'),
    prefix: tr(language, 'profilePrefix'),
    use: tr(language, 'useProfile'),
    invalid: tr(language, 'invalidProfile'),
  };
  const [profilePrefix, setProfilePrefix] = useState('');
  const [error, setError] = useState('');

  function useEnteredProfile() {
    if (!/^\d+$/.test(profilePrefix)) {
      setError(text.invalid);
      return;
    }
    setError('');
    onSelect(profilePrefix);
  }

  return (
    <section aria-labelledby="profiles-heading">
      <p className="eyebrow">{text.eyebrow}</p>
      <h2 id="profiles-heading">{text.title}</h2>
      <p className="muted-copy">{text.help}</p>

      {profiles.length > 0 && (
        <fieldset>
          <legend>{text.saved}</legend>
          {profiles.map((profileId) => (
            <label key={profileId}>
              <input
                checked={profileId === activeProfile}
                name="tracker-profile"
                onChange={() => onSelect(profileId)}
                type="radio"
              />
              {profileId}
            </label>
          ))}
        </fieldset>
      )}

      <label htmlFor="profile-prefix">{text.prefix}</label>
      <input
        id="profile-prefix"
        inputMode="numeric"
        onChange={(event) => setProfilePrefix(event.currentTarget.value)}
        type="text"
        value={profilePrefix}
      />
      {error && <p role="alert">{error}</p>}
      <button onClick={useEnteredProfile} type="button">{text.use}</button>
    </section>
  );
}
