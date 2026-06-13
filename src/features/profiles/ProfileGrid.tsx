import type { DeviceCapabilities } from "../../domain/device";
import type { LightingProfile } from "../../domain/profile";
import { LoadingState } from "../../shared/components/LoadingState";
import { ProfileCard } from "./ProfileCard";

interface ProfileGridProps {
  profiles: LightingProfile[];
  capabilities: DeviceCapabilities | null;
  loading: boolean;
  onApply: (profileId: string) => void;
}

export function ProfileGrid({
  profiles,
  capabilities,
  loading,
  onApply,
}: ProfileGridProps) {
  if (loading && profiles.length === 0) {
    return <LoadingState label="Loading built-in profiles" />;
  }

  return (
    <div className="profile-grid">
      {profiles.map((profile) => (
        <ProfileCard
          capabilities={capabilities}
          key={profile.id}
          loading={loading}
          profile={profile}
          onApply={onApply}
        />
      ))}
    </div>
  );
}
