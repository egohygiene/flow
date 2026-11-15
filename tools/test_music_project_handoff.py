#!/usr/bin/env python3
"""The private handoff observer cannot turn missing reviews into approval."""
import copy
import unittest
import check_music_project_handoff as project


class Cases(unittest.TestCase):
    def test_qc_does_not_substitute_presented_extent_or_wrong_subject(self):
        # Synthetic record shapes only; no meter or provider is executed here.
        artifact={'sha256':'a'*64,'byte_count':123}
        measurement=lambda unit:{'unit':unit,'value':{'kind':'measured','value':-20.0}}
        unavailable={'value':{'kind':'unavailable','reason':'unsupported_native_signal_profile'}}
        signal={'source_sha256':artifact['sha256'],'source_bytes':123,'provider_sha256':'b'*64,
            'ffmpeg_sha256':project.ANI_TOOLS['ffmpeg'],'ffprobe_sha256':project.ANI_TOOLS['ffprobe'],
            'frames':96256,'sample_rate_hz':48000,'channels':2,'normalization':'none','runtime_qualified':False,
            'decoded_pcm_sha256':'b70f04f7cd3c64fbb6c80784e69f5d10ef29c5df3643695dfe04be47d0c2a4ee',
            'decoded_wav_sha256':'c'*64,'decoded_wav_bytes':4096,
            'decoded_audio_tail':{'decoded_sample_frames':96256,'presented_sample_frames':96000,'nonpresented_tail_sample_frames':256,'metric_scope':'complete_decoded_pcm_including_nonpresented_tail'}}
        signal['loudness']={'frames':96256,'sample_rate_hz':48000,'channels':2,'pcm_sha256':signal['decoded_pcm_sha256'],
            'runtime_qualified':False,'integrated_loudness':measurement('loudness_units_full_scale'),'true_peak':measurement('decibels_true_peak')}
        value={'schema':'aniflow.final-audio-qc/v1','intent':{'artifact':artifact},'signal':signal,'normalization':'none',
            'runtime_qualified':False,'creative_approval':'not_performed; measurements are not musical quality review','publication_authority':'none',
            'accepted_pcm':{'source':{'frame_count':96256,'artifact':{'sha256':'c'*64,'byte_size':4096}},'integrated_loudness':copy.deepcopy(unavailable),'true_peak':copy.deepcopy(unavailable)}}
        project.qc_subject(value,artifact,'b'*64)
        for change in [lambda v:v['signal']['loudness'].update(frames=96000),lambda v:v['signal'].update(source_sha256='c'*64),
                lambda v:v['accepted_pcm']['integrated_loudness']['value'].update(kind='measured',value=-20.0),
                lambda v:v['accepted_pcm']['source']['artifact'].update(sha256='d'*64)]:
            altered=copy.deepcopy(value);change(altered)
            with self.assertRaises(ValueError):project.qc_subject(altered,artifact,'b'*64)

    def test_pending_review_cannot_promote_supplied_identity_or_authority(self):
        pending = {"reviews": {"state": "review_pending", "pending": ["words", "timing", "visual", "audio", "final_package"], "changes_requested": [], "identity_authenticated": False},
            "publishing_authority": False, "current_platform_compliance_verified": False}
        project.pending(pending)
        for update in [{"publishing_authority": True}, {"current_platform_compliance_verified": True},
            {"reviews": {"state": "operator_declarations_supplied", "pending": [], "identity_authenticated": True}}]:
            changed = copy.deepcopy(pending); changed.update(update)
            with self.assertRaises(ValueError): project.pending(changed)
        changed=copy.deepcopy(pending);changed['reviews']['pending'].remove('visual')
        with self.assertRaises(ValueError):project.pending(changed)


if __name__ == "__main__": unittest.main()
