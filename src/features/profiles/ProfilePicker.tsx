import { useState } from 'react';

type Language = 'eng' | 'fra';

const copy = {
  eng: { eyebrow: 'LOCAL PROFILES', title: 'Tracker profile', help: 'Profiles are stored only in Mistria Tracker. Choose a save prefix yourself.', saved: 'Saved tracker profiles', prefix: 'Profile prefix', use: 'Use profile', invalid: 'Enter a numeric profile prefix.' },
  fra: { eyebrow: 'PROFILS LOCAUX', title: 'Profil du suivi', help: 'Les profils sont enregistrés uniquement dans Mistria Tracker. Choisissez vous-même un préfixe de sauvegarde.', saved: 'Profils du suivi enregistrés', prefix: 'Préfixe du profil', use: 'Utiliser ce profil', invalid: 'Entrez un préfixe de profil numérique.' },
} as const;

type ProfilePickerProps = {
  activeProfile: string | null;
  language?: Language;
  profiles: string[];
  onSelect: (profileId: string) => void;
};

export function ProfilePicker({ activeProfile, language = 'eng', profiles, onSelect }: ProfilePickerProps) {
  const text = copy[language];
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
