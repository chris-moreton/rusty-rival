import importlib.util,json,os,sys,tempfile,unittest
from unittest import mock
from pathlib import Path
spec=importlib.util.spec_from_file_location('build_pgo',str(Path(__file__).with_name('build_pgo.py')));build=importlib.util.module_from_spec(spec);spec.loader.exec_module(build)

class BuilderTests(unittest.TestCase):
 def test_canary_metadata_must_match_hashed_log(self):
  root=Path(__file__).parent/'pgo'
  canary=json.loads((root/'diagnostic-canary.json').read_text())
  log=(root/'diagnostic-canary.log').read_text()
  self.assertTrue(build.validate_canary_log(canary,log)['passed'])
  for key,value in [('all_missing_count',2),('missing_search_diagnostics',[]),('exit_code',1)]:
   with self.subTest(key=key),self.assertRaises(RuntimeError):
    build.validate_canary_log(dict(canary,**{key:value}),log)
  for altered in [log.replace('6search6search','6search7closure'),log.replace('Finished `release` profile','unfinished')]:
   with self.assertRaises(RuntimeError):build.validate_canary_log(canary,altered)

 def test_frozen_training_has_no_holdout_or_bench_leakage(self):
  data=json.loads((Path(__file__).parent/'pgo/positions.json').read_text())
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'positions.json';p.write_text(json.dumps(data));self.assertEqual(len(build.load_training(p)),80)
   data['heldout'].append(data['training'][0]);p.write_text(json.dumps(data))
   with self.assertRaisesRegex(ValueError,'overlap'):build.load_training(p)
 def test_duplicate_training_rejected(self):
  data=json.loads((Path(__file__).parent/'pgo/positions.json').read_text());data['training'][1]=data['training'][0]
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'positions.json';p.write_text(json.dumps(data))
   with self.assertRaisesRegex(ValueError,'distinct'):build.load_training(p)

 def test_profile_mismatch_and_missing_hot_function_are_rejected(self):
  for line in ['warning: function control flow change detected (hash mismatch) foo', 'warning: no profile data available for function _RNvCxyz11rusty_rival6search6search', 'warning: no profile data available for function _RNvCxyz11rusty_rival10cold_setup']:
   with self.subTest(line=line),self.assertRaises(RuntimeError):build.validate_profile_warnings(build.profile_warnings(line))
  self.assertEqual(build.profile_warnings('warning: no profile data available for function cold_setup')['missing_function_count'],1)
  self.assertEqual(build.profile_warnings('warning: no profile data available for function _RNvCxyz5regex6search')['engine_missing_function_lines'],[])
  self.assertEqual(build.profile_warnings('Finished release profile')['missing_function_count'],0)

 def counts(self,rows):
  return ''.join('  '+name+':\n    Hash: 0x123\n    Counters: '+str(len(values))+'\n    Block counts: ['+', '.join(map(str,values))+']\n' for name,values in rows)
 # v0 names as the NET-1762 release binary profiles them: the orthodox
 # (`Kb0_`) instantiations are hot, the wrappers and `Kb1_` forms are not
 SEARCH_FALSE='_RINvNtCsTest_11rusty_rival6search11search_c960Kb0_EB4_'
 QUIESCE_FALSE='_RINvNtCsTest_11rusty_rival7quiesce12quiesce_c960Kb0_EB4_'
 NNUE='_RNvMs_NtCsTest_11rusty_rival4nnueNtB4_11Accumulator23update_accumulator_from'
 def test_hot_profile_requires_each_direct_function_with_positive_counts(self):
  rows=[(self.SEARCH_FALSE,[0,10]),(self.QUIESCE_FALSE,[0,10]),(self.NNUE,[0,10]),
        ('_RNvNtCsTest_11rusty_rival6search6search',[0,0]),('_RNvNtCsTest_11rusty_rival7quiesce7quiesce',[0,0])]
  records=build.audit_hot_counts(self.counts(rows))
  self.assertEqual(sorted(records),['11search_c960Kb0_E','12quiesce_c960Kb0_E','23update_accumulator_from'])
  self.assertEqual(records['11search_c960Kb0_E'][0]['symbol'],self.SEARCH_FALSE)
  with self.assertRaisesRegex(RuntimeError,'No positive'):build.audit_hot_counts('')
  with self.assertRaisesRegex(RuntimeError,'23update_accumulator_from'):
   build.audit_hot_counts(self.counts([(self.SEARCH_FALSE,[5]),(self.QUIESCE_FALSE,[5]),(self.NNUE+'_missing',[5])]))
 def test_hot_profile_rejects_wrappers_and_chess960_forms_in_place_of_the_orthodox_form(self):
  # The pre-NET-1762 names, or the Chess960 (`Kb1_`) instantiation, must not
  # satisfy the check: only the form the release binary runs counts
  for search in ['_RNvNtCsTest_11rusty_rival6search6search','_RINvNtCsTest_11rusty_rival6search11search_c960Kb1_EB4_']:
   with self.subTest(search=search),self.assertRaisesRegex(RuntimeError,'11search_c960Kb0_E'):
    build.audit_hot_counts(self.counts([(search,[9]),(self.QUIESCE_FALSE,[9]),(self.NNUE,[9])]))
  with self.assertRaisesRegex(RuntimeError,'12quiesce_c960Kb0_E'):
   build.audit_hot_counts(self.counts([(self.SEARCH_FALSE,[9]),('_RNvNtCsTest_11rusty_rival7quiesce7quiesce',[9]),(self.NNUE,[9])]))
  # Present but never executed is still a failure
  with self.assertRaisesRegex(RuntimeError,'11search_c960Kb0_E'):
   build.audit_hot_counts(self.counts([(self.SEARCH_FALSE,[0,0]),(self.QUIESCE_FALSE,[9]),(self.NNUE,[9])]))
 def test_hot_profile_ignores_closures_and_sorts_inside_the_orthodox_form(self):
  # A hot closure or std sort instantiated inside search_c960::<false> must
  # not stand in for the function itself; the `<cgu>;` prefix is accepted
  closure='rusty_rival.0123-cgu.0;_RINvMs_NtCsTest_8arrayvec8arrayvecINtB5_8ArrayVecmKj100_E6retainNCINvNtCsTest_11rusty_rival6search11search_c960Kb0_Es5_0EB1e_'
  sort='_RINvNtNtCsTest_4core5slice4sortTmlENCINvNtCsTest_11rusty_rival7quiesce12quiesce_c960Kb0_Es_0E0EB2c_'
  with self.assertRaisesRegex(RuntimeError,'11search_c960Kb0_E'):
   build.audit_hot_counts(self.counts([(self.SEARCH_FALSE,[0]),(closure,[9]),(self.QUIESCE_FALSE,[9]),(self.NNUE,[9])]))
  with self.assertRaisesRegex(RuntimeError,'12quiesce_c960Kb0_E'):
   build.audit_hot_counts(self.counts([(self.SEARCH_FALSE,[9]),(self.QUIESCE_FALSE,[0]),(sort,[9]),(self.NNUE,[9])]))
  records=build.audit_hot_counts(self.counts([('rusty_rival.0123-cgu.0;'+self.SEARCH_FALSE,[9]),(closure,[9]),(self.QUIESCE_FALSE,[9]),(self.NNUE,[9])]))
  self.assertEqual([r['symbol'] for r in records['11search_c960Kb0_E']],['rusty_rival.0123-cgu.0;'+self.SEARCH_FALSE])

 def test_current_source_bench_overlap_is_rejected(self):
  data=json.loads((Path(__file__).parent/'pgo/positions.json').read_text())
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp);(root/'src').mkdir();p=root/'positions.json';p.write_text(json.dumps(data))
   (root/'src/uci_bench.rs').write_text('const BENCH_FENS: [&str; 1] = ["'+data['training'][0]['fen']+'"];')
   with self.assertRaisesRegex(ValueError,'CURRENT source bench'):build.load_training(p,root)
 def test_runner_must_support_avx2_and_f16c(self):
  flags='avx avx2 bmi1 bmi2 fma movbe f16c xsave pni ssse3 sse4_1 sse4_2 popcnt cx16 lahf_lm abm'
  self.assertEqual(build.check_runner_isa('flags : '+flags)['logical_cpus_checked'],1)
  for flag in ['avx2','f16c','abm']:
   with self.subTest(flag=flag),self.assertRaisesRegex(RuntimeError,'lacks'):
    build.check_runner_isa('flags : '+' '.join(x for x in flags.split() if x!=flag))
 def fake(self,root,mode):
  p=root/'engine';p.write_text('#!'+sys.executable+'\n'+f'''import sys
from pathlib import Path
for line in sys.stdin:
 c=line.strip()
 if c=='uci':print('id name Rusty Rival {'wrong' if mode=='version' else '1.0.68'}\\nuciok',flush=True)
 elif c=='isready':print('readyok',flush=True)
 elif c.startswith('go'):
  print({'Error: rejected go' if mode=='error' else 'info depth 4 score mate 1 nodes 42 time 1 pv e2e4'+chr(10)+'bestmove e2e4'!r},flush=True)
 elif c=='quit':Path({str(root/'quit')!r}).write_text('yes');break
''');p.chmod(0o755);return p
 def test_short_mating_training_is_recorded_and_engine_quits(self):
  with tempfile.TemporaryDirectory() as tmp:
   r=Path(tmp);p=self.fake(r,'ok');rows=build.train(p,[{'fen':'8/8/8/8/8/8/4P3/K6k w - - 0 1'}],r,'1.0.68',r/'record.json')
   self.assertEqual(rows[0]['reported_nodes'],42);self.assertTrue((r/'quit').exists())
 def test_wrong_version_and_protocol_error_close_engine(self):
  for mode,pattern in [('version','wrong UCI version'),('error','rejected training command')]:
   with self.subTest(mode=mode),tempfile.TemporaryDirectory() as tmp:
    r=Path(tmp);p=self.fake(r,mode)
    with self.assertRaisesRegex(RuntimeError,pattern):build.train(p,[{'fen':'8/8/8/8/8/8/4P3/K6k w - - 0 1'}],r,'1.0.68',r/'record.json')
    self.assertTrue((r/'quit').exists())
 def test_silent_engine_timeout_names_awaited_response(self):
  with tempfile.TemporaryDirectory() as tmp:
   r=Path(tmp);p=self.fake(r,'ok')
   with mock.patch.object(build.queue.Queue,'get',side_effect=build.queue.Empty):
    with self.assertRaisesRegex(TimeoutError,'Engine did not send uciok'):
     build.train(p,[],r,'1.0.68',r/'record.json')
   self.assertTrue((r/'quit').exists())
if __name__=='__main__':unittest.main()
