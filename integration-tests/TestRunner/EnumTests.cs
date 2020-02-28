using System;
using System.Linq;
using Xunit;

namespace TestRunner
{
    public class EnumTests
    {
        [Fact]
        public void SimpleEnumRoundTrip()
        {
            foreach (var variant in Enum.GetValues(typeof(SimpleCEnum)).Cast<SimpleCEnum>())
            {
                SimpleCEnum result = IntegrationTests.RoundtripSimpleEnum(variant);
                Assert.Equal(variant, result);
            }
        }

        [Fact]
        public void DiscriminantEnumRoundTrip()
        {

            foreach (var variant in Enum.GetValues(typeof(EnumWithDiscriminants)).Cast<EnumWithDiscriminants>())
            {
                EnumWithDiscriminants result = IntegrationTests.RoundtripSimpleEnumWithDiscriminants(variant);
                Assert.Equal(variant, result);
            }
        }
    }
}
