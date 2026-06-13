import type { DeviceInfo } from "../../domain/device";
import type { LightingProfile } from "../../domain/profile";
import { Notice } from "../../shared/components/Notice";
import { ProfileGrid } from "./ProfileGrid";

interface ProfilesPageProps {
  device: DeviceInfo | null;
  profiles: LightingProfile[];
  loading: boolean;
  onApplyProfile: (profileId: string) => void;
}

export function ProfilesPage({
  device,
  profiles,
  loading,
  onApplyProfile,
}: ProfilesPageProps) {
  return (
    <section className="page-stack">
      <div className="page-heading">
        <div>
          <p className="eyebrow">Profiles</p>
          <h1>Built-in lighting profiles</h1>
        </div>
      </div>
      <Notice>
        Profiles are provided by the Rust backend. Custom persistence will be added
        later through the storage layer.
      </Notice>
      <ProfileGrid
        capabilities={device?.capabilities ?? null}
        loading={loading}
        profiles={profiles}
        onApply={onApplyProfile}
      />
    </section>
  );
}
